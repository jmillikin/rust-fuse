// Copyright 2020 John Millikin and the rust-fuse contributors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

//! An implementation of the FUSE protocol in Rust.

#![no_std]

#![allow(
	clippy::collapsible_if,
	clippy::if_same_then_else,
	clippy::len_without_is_empty,
	clippy::match_like_matches_macro,
	clippy::needless_late_init,
	clippy::needless_lifetimes,
	clippy::new_without_default,
	clippy::tabs_in_doc_comments,
)]

#![warn(
	// API hygiene
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::must_use_candidate,

	// Panic hygiene
	clippy::expect_used,
	clippy::todo,
	clippy::unimplemented,
	clippy::unwrap_used,

	// no_std hygiene
	clippy::std_instead_of_alloc,
	clippy::std_instead_of_core,

	// Documentation coverage
	missing_docs,
	clippy::missing_panics_doc,

	// Explicit casts
	clippy::fn_to_numeric_cast_any,
	clippy::ptr_as_ptr,

	// Optimization
	clippy::trivially_copy_pass_by_ref,

	// Unused symbols
	clippy::let_underscore_must_use,
	clippy::no_effect_underscore_binding,
	clippy::used_underscore_binding,

	// Leftover debugging
	clippy::print_stderr,
	clippy::print_stdout,
)]

#[macro_use]
mod internal;

mod node_id;
pub use node_id::NodeId;

mod file_mode;
pub use file_mode::{FileMode, FileType};

mod attributes;
pub use attributes::{Attributes};
pub(crate) use attributes::FuseAttrOut;

mod entry;
pub use entry::Entry;

/// Types and constants defined by the FUSE kernel interface.
///
/// This module is automatically generated from [`fuse.h`] in the Linux kernel
/// source tree.
///
/// [`fuse.h`]: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/linux/fuse.h?h=v5.19
#[allow(
	dead_code,
	missing_docs,
	non_camel_case_types,
	unused_parens,
	clippy::exhaustive_structs,
)]
pub mod kernel;

mod notify;
pub use notify::{
	FuseNotification,
	Delete as NotifyDelete,
	InvalidateEntry as NotifyInvalidateEntry,
	InvalidateInode as NotifyInvalidateInode,
	Poll as NotifyPoll,
};

pub(crate) mod lock;
pub use lock::{
	Lock,
	LockError,
	LockMode,
	LockRange,
	LockOwner,
	ProcessId as LockOwnerProcessId,
};

mod unix_time;
pub use unix_time::UnixTime;

// FIXME
pub use crate::os::{
	NodeName,
	NodeNameError,
	XattrName,
	XattrNameError,
	XattrValue,
	XattrValueError,
};

pub mod client;
pub mod cuse;
pub mod io;
pub mod operations;
pub mod os;
pub mod server;

/// The error type for FUSE operations.
///
/// The FUSE protocol represents errors as 32-bit signed integers, but the
/// client implementation in Linux rejects error numbers outside the
/// interval `[1, 512)`. Other implementations impose similar limits.
///
/// To provide an ergonomic API it must be possible for the user to pass
/// OS-specific error numbers to FUSE functions, but the size and signedness
/// of these error numbers varies between OSes. This means it's not appropriate
/// to use `Into<NonZeroI32>` trait bounds for error-related functions.
///
/// The `Error` type solves this by providing an unambiguous and
/// platform-independent encoding of error values, with the modules under
/// [`fuse::os`](crate::os) mapping OS-specific error codes into FUSE errors.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Error(pub core::num::NonZeroI32);

// FIXME
#[cfg(target_os = "freebsd")]
use crate::os::freebsd::OsError;
#[cfg(target_os = "linux")]
use crate::os::linux::OsError;

impl Error {
	pub(crate) const E2BIG: Error = OsError::E2BIG;
	pub(crate) const NOT_FOUND: Error = OsError::NOT_FOUND;
	pub(crate) const PROTOCOL_ERROR: Error = OsError::PROTOCOL_ERROR;
	pub(crate) const UNIMPLEMENTED: Error = OsError::UNIMPLEMENTED;
	pub(crate) const INVALID_ARGUMENT: Error = OsError::INVALID_ARGUMENT;
	pub(crate) const OVERFLOW: Error = OsError::OVERFLOW;
}

// RequestHeader {{{

/// The header of a FUSE request.
#[derive(Clone, Copy)]
pub struct RequestHeader(kernel::fuse_in_header);

impl RequestHeader {
	/// Returns the raw [`fuse_in_header`] for this request.
	///
	/// [`fuse_in_header`]: kernel::fuse_in_header
	#[inline]
	#[must_use]
	pub fn raw(&self) -> &kernel::fuse_in_header {
		&self.0
	}

	/// Returns the unique ID for this request.
	#[inline]
	#[must_use]
	pub fn request_id(&self) -> core::num::NonZeroU64 {
		unsafe { core::num::NonZeroU64::new_unchecked(self.0.unique) }
	}

	/// Returns the length of this request, including the header.
	#[inline]
	#[must_use]
	pub fn request_len(&self) -> core::num::NonZeroU32 {
		unsafe { core::num::NonZeroU32::new_unchecked(self.0.len) }
	}

	/// Returns the opcode of this request.
	#[inline]
	#[must_use]
	pub fn opcode(&self) -> kernel::fuse_opcode {
		self.0.opcode
	}

	/// Returns the ID of this request's primary node, if present.
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> Option<NodeId> {
		NodeId::new(self.0.nodeid)
	}

	/// Returns the user ID of the process that initiated this request.
	#[inline]
	#[must_use]
	pub fn user_id(&self) -> u32 {
		self.0.uid
	}

	/// Returns the group ID of the process that initiated this request.
	#[inline]
	#[must_use]
	pub fn group_id(&self) -> u32 {
		self.0.gid
	}

	/// Returns the process ID of the process that initiated this request,
	/// if present.
	///
	/// The concept of a "process ID" is not fully specified by POSIX, and some
	/// platforms may report process IDs that don't match the intuitive userland
	/// meaning. For example, platforms that represent processes as a group of
	/// threads might populate a request's process ID from the thread ID (TID)
	/// rather than the thread group ID (TGID).
	///
	/// A request might not have a process ID, for example if it was generated
	/// internally by the kernel, or if the client's PID isn't visible in the
	/// server's PID namespace.
	#[inline]
	#[must_use]
	pub fn process_id(&self) -> Option<core::num::NonZeroU32> {
		core::num::NonZeroU32::new(self.0.pid)
	}
}

impl core::fmt::Debug for RequestHeader {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		fmt.debug_struct("RequestHeader")
			.field("request_id", &self.request_id())
			.field("request_len", &self.request_len())
			.field("opcode", &self.opcode())
			.field("node_id", &format_args!("{:?}", self.node_id()))
			.field("user_id", &self.user_id())
			.field("group_id", &self.group_id())
			.field("process_id", &format_args!("{:?}", self.process_id()))
			.finish()
	}
}

// }}}

// ResponseHeader {{{

/// The header of a FUSE response.
#[derive(Copy, Clone)]
pub struct ResponseHeader {
	raw: kernel::fuse_out_header,
}

const HEADER_LEN_U32: u32 =
	core::mem::size_of::<kernel::fuse_out_header>() as u32;

impl ResponseHeader {
	/// Creates a new `ResponseHeader` with the given request ID.
	///
	/// The initial length is `size_of::<ResponseHeader>()`.
	#[inline]
	#[must_use]
	pub fn new(request_id: core::num::NonZeroU64) -> ResponseHeader {
		Self {
			raw: kernel::fuse_out_header {
				len: HEADER_LEN_U32,
				unique: request_id.get(),
				error: 0,
			},
		}
	}

	/// Creates a new `ResponseHeader` with no request ID (a notification).
	///
	/// The initial length is `size_of::<ResponseHeader>()`.
	#[inline]
	#[must_use]
	pub fn new_notification() -> ResponseHeader {
		Self {
			raw: kernel::fuse_out_header {
				len: HEADER_LEN_U32,
				unique: 0,
				error: 0,
			},
		}
	}

	/// Returns the unique ID for the original request, if present.
	///
	/// Responses without a request ID are notifications.
	#[inline]
	#[must_use]
	pub fn request_id(&self) -> Option<core::num::NonZeroU64> {
		core::num::NonZeroU64::new(self.raw.unique)
	}

	/// Returns the length of this response, including the header.
	#[inline]
	#[must_use]
	pub fn response_len(&self) -> core::num::NonZeroU32 {
		unsafe { core::num::NonZeroU32::new_unchecked(self.raw.len) }
	}

	/// Sets the length of this response, including the header.
	#[inline]
	pub fn set_response_len(&mut self, response_len: core::num::NonZeroU32) {
		self.raw.len = response_len.get();
	}

	/// Returns the error code for the original request, if present.
	///
	/// Responses without an error code indicate successful completion of the
	/// original request.
	#[inline]
	#[must_use]
	pub fn error(&self) -> Option<core::num::NonZeroI32> {
		core::num::NonZeroI32::new(self.raw.error)
	}

	/// Sets the error code for the original request.
	#[inline]
	pub fn set_error(&mut self, error: core::num::NonZeroI32) {
		self.raw.error = error.get();
	}
}

impl core::fmt::Debug for ResponseHeader {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		fmt.debug_struct("ResponseHeader")
			.field("request_id", &format_args!("{:?}", self.request_id()))
			.field("response_len", &self.response_len())
			.field("error", &format_args!("{:?}", self.error()))
			.finish()
	}
}

// }}}

/// OS-specific flags passed to `fallocate()`.
pub type FallocateFlags = u32;

/// OS-specific flags passed to `open()`.
pub type OpenFlags = u32;

/// OS-specific flags passed to `renameat2()`.
pub type RenameFlags = u32;

/// OS-specific flags passed to `setxattr()`.
pub type SetxattrFlags = u32;

/// OS-specific event types used with `poll()`.
pub type PollEvents = u32;

// Version {{{

/// A version of the FUSE protocol.
///
/// FUSE protocol versions are a (major, minor) version tuple, but FUSE does
/// not use these terms in their common meaning. Backwards compatibility is
/// freely broken in "minor" releases, and the major version is only used
/// during initial connection setup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
	major: u32,
	minor: u32,
}

impl Version {
	const LATEST: Version = Version {
		major: kernel::FUSE_KERNEL_VERSION,
		minor: kernel::FUSE_KERNEL_MINOR_VERSION,
	};

	/// Create a new `Version` with the given major and minor version numbers.
	#[inline]
	#[must_use]
	pub const fn new(major: u32, minor: u32) -> Version {
		Version { major, minor }
	}

	/// Return the versions's major version number.
	#[inline]
	#[must_use]
	pub const fn major(&self) -> u32 {
		self.major
	}

	/// Return the versions's major version number.
	#[inline]
	#[must_use]
	pub const fn minor(&self) -> u32 {
		self.minor
	}
}

// }}}
