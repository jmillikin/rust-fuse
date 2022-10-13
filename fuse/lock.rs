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
use crate::internal::fuse_kernel;

#[cfg(target_os = "freebsd")]
mod sys_fcntl {
	pub(crate) const F_RDLCK: u32 = 1;
	pub(crate) const F_UNLCK: u32 = 2;
	pub(crate) const F_WRLCK: u32 = 3;
}

#[cfg(all(
	target_os = "linux",
	target_arch = "alpha",
))]
mod sys_fcntl {
	pub(crate) const F_RDLCK: u32 = 1;
	pub(crate) const F_WRLCK: u32 = 2;
	pub(crate) const F_UNLCK: u32 = 8;
}

#[cfg(all(
	target_os = "linux",
	any(
		target_arch = "sparc",
		target_arch = "parisc",
	),
))]
mod sys_fcntl {
	pub(crate) const F_RDLCK: u32 = 1;
	pub(crate) const F_WRLCK: u32 = 2;
	pub(crate) const F_UNLCK: u32 = 3;
}

#[cfg(all(
	target_os = "linux",
	not(any(
		target_arch = "alpha",
		target_arch = "sparc",
		target_arch = "parisc",
	)),
))]
mod sys_fcntl {
	pub(crate) const F_RDLCK: u32 = 0;
	pub(crate) const F_WRLCK: u32 = 1;
	pub(crate) const F_UNLCK: u32 = 2;
}

pub(crate) use sys_fcntl::{F_RDLCK, F_UNLCK, F_WRLCK};

pub(crate) const OFFSET_MAX: u64 = i64::MAX as u64;

// Owner {{{

/// Opaque identifier for the owner of advisory locks.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Owner {
	owner: u64,
}

impl Owner {
	/// Creates a new `Owner` with the given lock owner.
	#[inline]
	#[must_use]
	pub fn new(owner: u64) -> Owner {
		Owner { owner }
	}

	/// Returns the lock owner as a primitive integer.
	#[inline]
	#[must_use]
	pub fn get(&self) -> u64 {
		self.owner
	}
}

impl fmt::Debug for Owner {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::hex_u64(self.owner).fmt(fmt)
	}
}

// }}}

// Range {{{

/// Represents a (possibly unbounded) range of bytes within a file.
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Range {
	start: u64,
	length: Option<num::NonZeroU64>,
}

impl Range {
	/// Creates a new `Range` with the given start offset and length.
	#[inline]
	#[must_use]
	pub const fn new(start: u64, length: Option<num::NonZeroU64>) -> Range {
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
}

impl fmt::Debug for Range {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Range")
			.field("start", &self.start())
			.field("length", &format_args!("{:?}", self.length()))
			.finish()
	}
}

// }}}

// ProcessId {{{

/// Represents a process that owns a POSIX-style advisory lock.
///
/// This type represents a different notion of "process ID" than the
/// [`fuse::ProcessId`](crate::ProcessId) struct, although on some platforms
/// they may be equivalent.
///
/// For representing lock ownership the [`Owner`] type should be used instead.
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
	pub fn get(&self) -> u32 {
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
	/// The lock mode is not `F_RDLCK` or `F_WRLCK`.
	UnknownMode,

	/// A record lock's range is zero-length.
	EmptyRange,
}

/// Whether a lock is an exclusive (write) or shared (read) lock.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Mode {
	/// Exclusive locks may be held only by a single owner.
	Exclusive,

	/// Shared locks may be held by any number of owners.
	Shared,
}

// Lock {{{

/// Represents an advisory lock on an open file.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Lock {
	mode: Mode,
	range: Range,
	process_id: Option<ProcessId>,
}

impl Lock {
	/// Creates a new `Lock` with the given mode, range, and process ID.
	#[inline]
	#[must_use]
	pub fn new(
		mode: Mode,
		range: Range,
		process_id: Option<ProcessId>,
	) -> Lock {
		Self { mode, range, process_id }
	}

	/// Returns the lock's mode (exclusive or shared).
	#[inline]
	#[must_use]
	pub fn mode(&self) -> Mode {
		self.mode
	}

	/// Returns the byte range covered by a lock.
	#[inline]
	#[must_use]
	pub fn range(&self) -> Range {
		self.range
	}

	/// Returns the process ID associated with a lock at construction.
	#[inline]
	#[must_use]
	pub fn process_id(&self) -> Option<ProcessId> {
		self.process_id
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

pub(crate) fn decode(
	raw: &fuse_kernel::fuse_file_lock,
) -> Result<Lock, LockError> {
	let mode = decode_mode(raw)?;
	let range = decode_range(raw)?;
	let process_id = ProcessId::new(raw.pid);
	Ok(Lock { mode, range, process_id })
}

pub(crate) fn decode_mode(
	raw: &fuse_kernel::fuse_file_lock,
) -> Result<Mode, LockError> {
	match raw.r#type {
		F_WRLCK => Ok(Mode::Exclusive),
		F_RDLCK => Ok(Mode::Shared),
		_ => Err(LockError::UnknownMode),
	}
}

pub(crate) fn decode_range(
	raw: &fuse_kernel::fuse_file_lock,
) -> Result<Range, LockError> {
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
			Some(length) => Ok(Range {
				start,
				length: Some(length),
			}),
		};
	}

	if raw.end >= OFFSET_MAX {
		return Ok(Range {
			start: raw.start,
			length: None,
		});
	}

	let length = (raw.end - raw.start).saturating_add(1);
	Ok(Range {
		start: raw.start,
		length: Some(unsafe {
			num::NonZeroU64::new_unchecked(length)
		}),
	})
}
