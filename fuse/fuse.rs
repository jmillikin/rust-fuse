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

#![cfg_attr(feature = "unstable_async", feature(async_fn_in_trait))]

#![allow(
	clippy::collapsible_if,
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

pub use crate::internal::fuse_kernel as kernel;

mod error;

mod node_id;
pub use node_id::NodeId;

mod node_name;
pub use node_name::{NodeName, NodeNameError};

mod file_mode;
pub use file_mode::{FileMode, FileType};

mod attributes;
pub use attributes::{Attributes};
pub(crate) use attributes::FuseAttrOut;

mod entry;
pub use entry::Entry;

mod xattr_name;
pub use xattr_name::{XattrName, XattrNameError};

mod xattr_value;
pub use xattr_value::{XattrValue, XattrValueError};

#[cfg(target_os = "linux")]
pub(crate) const XATTR_LIST_MAX: usize = 65536;

mod errno {
	#[cfg(target_os = "freebsd")]
	use freebsd_errno as os_errno;

	#[cfg(target_os = "linux")]
	use linux_errno as os_errno;

	use crate::Error;

	#[cfg(target_os = "linux")]
	pub(super) const ENODATA: Error = Error::from_errno(os_errno::ENODATA);

	#[cfg(target_os = "freebsd")]
	pub(super) const ENOATTR: Error = Error::from_errno(os_errno::ENOATTR);
}

#[cfg(target_os = "linux")]
macro_rules! enodata_or_enoattr {
	() => { errno::ENODATA };
}

#[cfg(target_os = "freebsd")]
macro_rules! enodata_or_enoattr {
	() => { errno::ENOATTR };
}

/// The requested extended attribute does not exist.
///
/// This error maps to either `ENODATA` or `ENOATTR`, depending on the
/// target platform.
pub const XATTR_NOT_FOUND: crate::Error = enodata_or_enoattr!();

pub mod client;
pub mod cuse;
pub mod io;
pub mod lock;
pub mod notify;
pub mod operations;
pub mod os;
pub mod server;
#[cfg(feature = "unstable_async")]
pub mod server_async;

pub use self::error::Error;

// RequestHeader {{{

/// The header of a FUSE request.
#[derive(Clone, Copy)]
pub struct RequestHeader {
	raw: kernel::fuse_in_header,
}

impl RequestHeader {
	/// Returns the unique ID for this request.
	#[inline]
	#[must_use]
	pub fn request_id(&self) -> core::num::NonZeroU64 {
		unsafe { core::num::NonZeroU64::new_unchecked(self.raw.unique) }
	}

	/// Returns the length of this request, including the header.
	#[inline]
	#[must_use]
	pub fn request_len(&self) -> core::num::NonZeroU32 {
		unsafe { core::num::NonZeroU32::new_unchecked(self.raw.len) }
	}

	/// Returns the opcode of this request.
	#[inline]
	#[must_use]
	pub fn opcode(&self) -> crate::kernel::fuse_opcode {
		self.raw.opcode
	}

	/// Returns the ID of this request's primary node, if present.
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> Option<crate::NodeId> {
		crate::NodeId::new(self.raw.nodeid)
	}

	/// Returns the user ID of the process that initiated this request.
	#[inline]
	#[must_use]
	pub fn user_id(&self) -> u32 {
		self.raw.uid
	}

	/// Returns the group ID of the process that initiated this request.
	#[inline]
	#[must_use]
	pub fn group_id(&self) -> u32 {
		self.raw.gid
	}

	/// Returns the process ID of the process that initiated this request,
	/// if present.
	///
	/// See the documentation of [`ProcessId`](crate::ProcessId) for details
	/// on the semantics of this value.
	///
	/// A request might not have a process ID, for example if it was generated
	/// internally by the kernel, or if the client's PID isn't visible in the
	/// server's PID namespace.
	#[inline]
	#[must_use]
	pub fn process_id(&self) -> Option<ProcessId> {
		ProcessId::new(self.raw.pid)
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

// ProcessId {{{

/// Represents a process that initiated a FUSE request.
///
/// The concept of a "process ID" is not fully specified by POSIX, and some
/// platforms may report process IDs that don't match the intuitive userland
/// meaning. For example, platforms that represent processes as a group of
/// threads might populate a request's process ID from the thread ID (TID)
/// rather than the thread group ID (TGID).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProcessId {
	pid: core::num::NonZeroU32,
}

impl ProcessId {
	/// Creates a new `ProcessId` if the given value is not zero.
	#[inline]
	#[must_use]
	pub fn new(pid: u32) -> Option<ProcessId> {
		Some(Self {
			pid: core::num::NonZeroU32::new(pid)?,
		})
	}

	/// Returns the process ID as a primitive integer.
	#[inline]
	#[must_use]
	pub fn get(&self) -> u32 {
		self.pid.get()
	}
}

impl core::fmt::Debug for ProcessId {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		self.pid.fmt(fmt)
	}
}

// }}}

// UnixTime {{{

/// A measurement of Unix time with nanosecond precision.
///
/// Unix time is the number of Unix seconds that have elapsed since the Unix
/// epoch of 1970-01-01 00:00:00 UTC. Unix seconds are exactly 1/86400 of a day.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnixTime {
	seconds: i64,
	nanos: u32,
}

const UNIX_EPOCH: UnixTime = UnixTime {
	seconds: 0,
	nanos: 0,
};

impl UnixTime {
	/// The Unix epoch, 1970-01-01 00:00:00 UTC.
	pub const EPOCH: UnixTime = UNIX_EPOCH;

	/// Creates a new `UnixTime` with the given offset from the epoch.
	///
	/// Returns `None` if the nanoseconds value exceeds 999,999,999.
	#[inline]
	#[must_use]
	pub const fn new(seconds: i64, nanos: u32) -> Option<UnixTime> {
		if nanos > crate::internal::timestamp::MAX_NANOS {
			return None;
		}
		Some(Self { seconds, nanos })
	}

	/// Creates a new `UnixTime` without checking that the nanoseconds value
	/// is valid.
	///
	/// # Safety
	///
	/// The nanoseconds value must not exceed 999,999,999.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(seconds: i64, nanos: u32) -> UnixTime {
		Self { seconds, nanos }
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_timespec_unchecked(
		seconds: u64,
		nanos: u32,
	) -> UnixTime {
		Self {
			seconds: seconds as i64,
			nanos,
		}
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_timespec(&self) -> (u64, u32) {
		(self.seconds as u64, self.nanos)
	}

	/// Returns the number of whole seconds contained by this `UnixTime`.
	#[inline]
	#[must_use]
	pub const fn seconds(&self) -> i64 {
		self.seconds
	}

	/// Returns the fractional part of this `UnixTime`, in nanoseconds.
	#[inline]
	#[must_use]
	pub const fn nanos(&self) -> u32 {
		self.nanos
	}
}

impl core::fmt::Debug for UnixTime {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		fmt.debug_tuple("UnixTime")
			.field(&format_args!("{:?}.{:09?}", self.seconds, self.nanos))
			.finish()
	}
}

// }}}

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
