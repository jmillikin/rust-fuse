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

#![cfg_attr(not(any(doc, feature = "std")), no_std)]

#![cfg_attr(feature = "unstable_async", feature(async_fn_in_trait))]

#![allow(
	clippy::collapsible_if,
	clippy::len_without_is_empty,
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
	clippy::std_instead_of_core,

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

mod error;

pub mod client;
pub mod cuse;
pub mod io;
pub mod lock;
pub mod node;
pub mod notify;
pub mod operations;
pub mod os;
pub mod server;
#[cfg(feature = "unstable_async")]
pub mod server_async;
pub mod xattr;

pub use self::error::Error;
pub use self::operations::types_only::*;

// RequestHeader {{{

/// The header of a FUSE request.
#[derive(Clone, Copy)]
pub struct RequestHeader {
	raw: internal::fuse_kernel::fuse_in_header,
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
	pub fn opcode(&self) -> Opcode {
		Opcode {
			bits: self.raw.opcode.0,
		}
	}

	/// Returns the ID of this request's primary node, if present.
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> Option<node::Id> {
		node::Id::new(self.raw.nodeid)
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
	raw: internal::fuse_kernel::fuse_out_header,
}

const HEADER_LEN_U32: u32 =
	core::mem::size_of::<internal::fuse_kernel::fuse_out_header>() as u32;

impl ResponseHeader {
	/// Creates a new `ResponseHeader` with the given request ID.
	///
	/// The initial length is `size_of::<ResponseHeader>()`.
	#[inline]
	#[must_use]
	pub fn new(request_id: core::num::NonZeroU64) -> ResponseHeader {
		Self {
			raw: internal::fuse_kernel::fuse_out_header {
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
			raw: internal::fuse_kernel::fuse_out_header {
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

// PollHandle {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PollHandle {
	bits: u64,
}

impl core::fmt::Debug for PollHandle {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		self.bits.fmt(fmt)
	}
}

// }}}

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

	/// Attempts to convert this `UnixTime` to a [`SystemTime`].
	///
	/// [`SystemTime`]: std::time::SystemTime
	#[cfg(any(feature = "std", doc))]
	#[must_use]
	pub fn to_system_time(&self) -> Option<std::time::SystemTime> {
		#![allow(clippy::std_instead_of_core)]
		use std::time::{Duration, SystemTime};

		if self.seconds == 0 && self.nanos == 0 {
			return Some(SystemTime::UNIX_EPOCH);
		}

		if self.seconds > 0 {
			return SystemTime::UNIX_EPOCH
				.checked_add(Duration::from_secs(self.seconds as u64))?
				.checked_add(Duration::from_nanos(u64::from(self.nanos)));
		}

		let seconds = self.seconds.checked_neg()?;
		SystemTime::UNIX_EPOCH
			.checked_sub(Duration::from_secs(seconds as u64))?
			.checked_sub(Duration::from_nanos(u64::from(self.nanos)))
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
		major: internal::fuse_kernel::FUSE_KERNEL_VERSION,
		minor: internal::fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
	};

	#[inline]
	#[must_use]
	pub const fn new(major: u32, minor: u32) -> Version {
		Version { major, minor }
	}

	#[inline]
	#[must_use]
	pub const fn major(&self) -> u32 {
		self.major
	}

	#[inline]
	#[must_use]
	pub const fn minor(&self) -> u32 {
		self.minor
	}
}

// }}}

// Opcode {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Opcode {
	bits: u32,
}

impl core::fmt::Debug for Opcode {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		internal::fuse_kernel::fuse_opcode(self.bits).fmt(fmt)
	}
}

macro_rules! export_opcodes {
	( $( $(#[$meta:meta])* $name:ident , )+ ) => {
		mod fuse_opcode {
			$(
				pub const $name: crate::Opcode = crate::Opcode {
					bits: crate::internal::fuse_kernel::$name.0,
				};
			)+
		}
		impl Opcode {
			$(
				$(#[$meta])*
				pub const $name: Opcode = fuse_opcode::$name;
			)+
		}
	};
}

export_opcodes! {
	FUSE_LOOKUP,
	FUSE_FORGET,
	FUSE_GETATTR,
	FUSE_SETATTR,
	FUSE_READLINK,
	FUSE_SYMLINK,
	FUSE_MKNOD,
	FUSE_MKDIR,
	FUSE_UNLINK,
	FUSE_RMDIR,
	FUSE_RENAME,
	FUSE_LINK,
	FUSE_OPEN,
	FUSE_READ,
	FUSE_WRITE,
	FUSE_STATFS,
	FUSE_RELEASE,
	FUSE_FSYNC,
	FUSE_SETXATTR,
	FUSE_GETXATTR,
	FUSE_LISTXATTR,
	FUSE_REMOVEXATTR,
	FUSE_FLUSH,
	FUSE_INIT,
	FUSE_OPENDIR,
	FUSE_READDIR,
	FUSE_RELEASEDIR,
	FUSE_FSYNCDIR,
	FUSE_GETLK,
	FUSE_SETLK,
	FUSE_SETLKW,
	FUSE_ACCESS,
	FUSE_CREATE,
	FUSE_INTERRUPT,
	FUSE_BMAP,
	FUSE_DESTROY,
	FUSE_IOCTL,
	FUSE_POLL,
	FUSE_NOTIFY_REPLY,
	FUSE_BATCH_FORGET,
	FUSE_FALLOCATE,
	FUSE_READDIRPLUS,
	FUSE_RENAME2,
	FUSE_LSEEK,
	FUSE_COPY_FILE_RANGE,
	FUSE_SETUPMAPPING,
	FUSE_REMOVEMAPPING,
	FUSE_SYNCFS,

	CUSE_INIT,

	CUSE_INIT_BSWAP_RESERVED,
	FUSE_INIT_BSWAP_RESERVED,
}

// }}}
