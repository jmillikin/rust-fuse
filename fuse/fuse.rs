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
pub mod lock;
pub mod node;
pub mod server;

pub mod os {
	#[cfg(any(doc, target_os = "freebsd"))]
	pub mod freebsd;

	#[cfg(not(any(doc, target_os = "freebsd")))]
	#[deprecated = "doc stub"]
	pub mod freebsd {
		#[deprecated = "doc stub"]
		pub struct MountOptions<'a> { _p: &'a () }
	}

	#[cfg(any(doc, target_os = "linux"))]
	pub mod linux;

	#[cfg(not(any(doc, target_os = "linux")))]
	#[deprecated = "doc stub"]
	pub mod linux {
		#[deprecated = "doc stub"]
		pub struct MountOptions<'a> { _p: &'a () }
	}
}

pub mod io;

pub use crate::error::Error;

#[macro_use]
mod protocol {
	#[macro_use]
	pub(crate) mod common;
}

pub mod notify;

pub mod operations;

pub use self::operations::types_only::*;

pub mod xattr;

pub const MIN_READ_BUFFER: usize = internal::fuse_kernel::FUSE_MIN_READ_BUFFER;

mod sealed {
	pub trait Sealed {}
}

pub trait Flags<Flag>: sealed::Sealed {
	fn new() -> Self;

	fn get(&self, flag: Flag) -> bool;

	fn set(&mut self, flag: Flag);
}

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

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PollHandle {
	bits: u64,
}

impl core::fmt::Debug for PollHandle {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		self.bits.fmt(fmt)
	}
}

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
		Some(Self { pid: core::num::NonZeroU32::new(pid)? })
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Opcode {
	bits: u32,
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
