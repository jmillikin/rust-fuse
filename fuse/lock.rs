// Copyright 2021 John Millikin and the rust-fuse contributors.
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

//! Advisory file and record locking.

use core::fmt;
use core::num;

use crate::internal::debug;
use crate::kernel;

pub(crate) const OFFSET_MAX: u64 = i64::MAX as u64;

/// Opaque identifier for the owner of advisory locks.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LockOwner(pub u64);

impl fmt::Debug for LockOwner {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::hex_u64(self.0).fmt(fmt)
	}
}

/// Represents a (possibly unbounded) range of bytes within a file.
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LockRange {
	start: u64,
	length: Option<num::NonZeroU64>,
}

impl LockRange {
	/// Creates a new `LockRange` with the given start offset and length.
	#[inline]
	#[must_use]
	pub const fn new(start: u64, length: Option<num::NonZeroU64>) -> LockRange {
		Self { start, length }
	}

	/// Returns the start offset.
	#[inline]
	#[must_use]
	pub const fn start(&self) -> u64 {
		self.start
	}

	/// Returns the length if the record is bounded.
	#[inline]
	#[must_use]
	pub const fn length(&self) -> Option<num::NonZeroU64> {
		self.length
	}

	/// Returns the end offset if the record is bounded.
	#[inline]
	#[must_use]
	pub const fn end(&self) -> Option<u64> {
		match self.length {
			None => None,
			Some(length) => Some(self.start.saturating_add(length.get() - 1)),
		}
	}

	pub(crate) fn decode(
		raw: &kernel::fuse_file_lock,
	) -> Result<LockRange, LockError> {
		// Both Linux and FreeBSD allow the `(*struct flock)->l_len` field to be
		// negative, but generate different `fuse_file_lock` values in this case:
		//
		// * Linux swaps the `start` and `end` fields before generating the
		//   FUSE request, such that the `end >= start` invariant is maintained.
		//
		// * FreeBSD leaves `start` unchanged and computes `end` relative to
		//   the negative length.
		//
		// To avoid exposing this to FUSE filesystem authors, detect the case of
		// `start > end` and swap the fields.
		if raw.start > raw.end {
			let start = raw.end.saturating_add(1);
			return match num::NonZeroU64::new(raw.start - start) {
				None => Err(LockError::EmptyRange),
				Some(length) => Ok(LockRange {
					start,
					length: Some(length),
				}),
			};
		}

		if raw.end >= OFFSET_MAX {
			return Ok(LockRange {
				start: raw.start,
				length: None,
			});
		}

		let length = (raw.end - raw.start).saturating_add(1);
		Ok(LockRange {
			start: raw.start,
			length: Some(unsafe {
				num::NonZeroU64::new_unchecked(length)
			}),
		})
	}
}

impl fmt::Debug for LockRange {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LockRange")
			.field("start", &self.start())
			.field("length", &format_args!("{:?}", self.length()))
			.finish()
	}
}

// }}}

// ProcessId {{{

/// Represents a process that owns a POSIX-style advisory lock.
///
/// The concept of a "process ID" is not fully specified by POSIX, and some
/// platforms may report process IDs that don't match the intuitive userland
/// meaning. For example, platforms that represent processes as a group of
/// threads might populate a request's process ID from the thread ID (TID)
/// rather than the thread group ID (TGID).
///
/// This type represents a different notion of "process ID" than the
/// [`RequestHeader::process_id`](crate::RequestHeader::process_id), although
/// on some platforms they may be equivalent.
///
/// For representing lock ownership the [`LockOwner`] type should be used
/// instead.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProcessId {
	pid: num::NonZeroU32,
}

impl ProcessId {
	/// Creates a new `ProcessId` if the given value is not zero.
	#[inline]
	#[must_use]
	pub fn new(pid: u32) -> Option<ProcessId> {
		Some(Self {
			pid: num::NonZeroU32::new(pid)?,
		})
	}

	/// Returns the process ID as a primitive integer.
	#[inline]
	#[must_use]
	pub fn get(self) -> u32 {
		self.pid.get()
	}
}

impl fmt::Debug for ProcessId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.pid.fmt(fmt)
	}
}

// }}}

/// Errors that may occur when validating a lock.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LockError {
	/// A record lock's range is zero-length.
	EmptyRange,
}

/// Whether a lock is an exclusive (write) or shared (read) lock.
///
/// The platform-specific constants `F_RDLCK`, `F_WRLCK`, and `F_UNLCK` may be
/// found in module `fuse::os::{target_os}`.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LockMode(pub u32);

#[cfg(target_os = "freebsd")]
const F_RDLCK: LockMode = crate::os::freebsd::F_RDLCK;

#[cfg(target_os = "freebsd")]
const F_WRLCK: LockMode = crate::os::freebsd::F_WRLCK;

#[cfg(target_os = "freebsd")]
const F_UNLCK: LockMode = crate::os::freebsd::F_UNLCK;

#[cfg(target_os = "linux")]
const F_RDLCK: LockMode = crate::os::linux::F_RDLCK;

#[cfg(target_os = "linux")]
const F_WRLCK: LockMode = crate::os::linux::F_WRLCK;

#[cfg(target_os = "linux")]
const F_UNLCK: LockMode = crate::os::linux::F_UNLCK;

impl fmt::Debug for LockMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			#[cfg(any(target_os = "freebsd", target_os = "linux"))]
			F_RDLCK => fmt.write_str("F_RDLCK"),
			#[cfg(any(target_os = "freebsd", target_os = "linux"))]
			F_WRLCK => fmt.write_str("F_WRLCK"),
			#[cfg(any(target_os = "freebsd", target_os = "linux"))]
			F_UNLCK => fmt.write_str("F_UNLCK"),
			_ => write!(fmt, "LockMode({})", self.0),
		}
	}
}

// Lock {{{

/// Represents an advisory lock on an open file.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Lock {
	mode: LockMode,
	range: LockRange,
	process_id: Option<ProcessId>,
}

impl Lock {
	/// Creates a new `Lock` with the given mode, range, and process ID.
	#[inline]
	#[must_use]
	pub fn new(
		mode: LockMode,
		range: LockRange,
		process_id: Option<ProcessId>,
	) -> Lock {
		Self { mode, range, process_id }
	}

	/// Returns the lock's mode (exclusive or shared).
	#[inline]
	#[must_use]
	pub fn mode(&self) -> LockMode {
		self.mode
	}

	/// Returns the byte range covered by a lock.
	#[inline]
	#[must_use]
	pub fn range(&self) -> LockRange {
		self.range
	}

	/// Returns the process ID associated with a lock at construction.
	#[inline]
	#[must_use]
	pub fn process_id(&self) -> Option<ProcessId> {
		self.process_id
	}

	pub(crate) fn decode(
		raw: &kernel::fuse_file_lock,
	) -> Result<Lock, LockError> {
		let mode = LockMode(raw.r#type);
		let range = LockRange::decode(raw)?;
		let process_id = ProcessId::new(raw.pid);
		Ok(Lock { mode, range, process_id })
	}
}

impl fmt::Debug for Lock {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Lock")
			.field("mode", &self.mode())
			.field("range", &self.range())
			.field("process_id", &format_args!("{:?}", self.process_id()))
			.finish()
	}
}

// }}}
