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

use core::fmt;
use core::num::NonZeroU64;

use crate::internal::fuse_kernel;

#[rustfmt::skip]
pub(crate) const F_RDLCK: u32 = {
	#[cfg(target_os = "linux")] {
		#[cfg(any(
			target_arch = "arm",
			target_arch = "x86",
			target_arch = "x86_64",
		))]
		{ 0 }
	}

	#[cfg(target_os = "freebsd")] { 1 }
};

#[rustfmt::skip]
pub(crate) const F_WRLCK: u32 = {
	#[cfg(target_os = "linux")] {
		#[cfg(any(
			target_arch = "arm",
			target_arch = "x86",
			target_arch = "x86_64",
		))]
		{ 1 }
	}

	#[cfg(target_os = "freebsd")] { 3 }
};

#[rustfmt::skip]
pub(crate) const F_UNLCK: u32 = {
	#[cfg(target_os = "linux")] {
		#[cfg(any(
			target_arch = "arm",
			target_arch = "x86",
			target_arch = "x86_64",
		))]
		{ 2 }
	}

	#[cfg(target_os = "freebsd")] { 2 }
};

#[rustfmt::skip]
const OFFSET_MAX: u64 = {
	#[cfg(target_arch = "x86_64")] {
		core::i64::MAX as u64
	}
};

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct LockRange {
	pub(crate) start: u64,
	pub(crate) end: u64,
}

impl LockRange {
	pub fn new(start: u64, length: Option<NonZeroU64>) -> Self {
		Self {
			start,
			end: match length {
				None => OFFSET_MAX,
				Some(len) => start + len.get() - 1, // TODO: clamp overflow
			},
		}
	}

	pub(crate) fn parse(raw: fuse_kernel::fuse_file_lock) -> Self {
		#[cfg(target_os = "freebsd")]
		{
			// Both Linux and FreeBSD allow the `(*struct flock)->l_len` field to be
			// negative, but generate different `fuse_file_lock` values in this case:
			//
			//   * Linux swaps the `start` and `end` fields before generating the
			//     FUSE request, such that the `end >= start` invariant is maintained.
			//   * FreeBSD leaves `start` unchanged and computes `end` relative to
			//     the negative length.
			//
			// To avoid exposing this to FUSE filesystem authors, when running under
			// FreeBSD detect the case of `start > end` and swap the fields.
			if raw.start > raw.end {
				return Self {
					start: raw.end + 1, // TODO: clamp overflow
					end: raw.start - 1,
				};
			}
		}
		Self {
			start: raw.start,
			end: raw.end,
		}
	}

	pub fn start(&self) -> u64 {
		self.start
	}

	pub fn end(&self) -> Option<u64> {
		if self.end == OFFSET_MAX {
			return None;
		}
		Some(self.end)
	}

	pub fn length(&self) -> Option<NonZeroU64> {
		if self.end == OFFSET_MAX {
			return None;
		}
		NonZeroU64::new((self.end - self.start) + 1) // TODO: clamp
	}
}

impl fmt::Debug for LockRange {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		if self.end == OFFSET_MAX {
			return write!(fmt, "{}..", self.start);
		}
		write!(fmt, "{}..{}", self.start, self.end + 1) // TODO: clamp
	}
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Lock {
	#[non_exhaustive]
	Shared { range: LockRange, process_id: u32 },

	#[non_exhaustive]
	Exclusive { range: LockRange, process_id: u32 },
}

impl Lock {
	pub fn new_shared(range: LockRange) -> Self {
		Self::Shared {
			range,
			process_id: 0,
		}
	}

	pub fn new_exclusive(range: LockRange) -> Self {
		Self::Exclusive {
			range,
			process_id: 0,
		}
	}

	pub fn range(&self) -> LockRange {
		match self {
			Self::Shared { range, .. } => *range,
			Self::Exclusive { range, .. } => *range,
		}
	}

	pub fn set_range(&mut self, range: LockRange) {
		let x = range;
		match self {
			Self::Shared { range, .. } => *range = x,
			Self::Exclusive { range, .. } => *range = x,
		}
	}

	pub fn process_id(&self) -> u32 {
		match self {
			Self::Shared { process_id, .. } => *process_id,
			Self::Exclusive { process_id, .. } => *process_id,
		}
	}

	pub fn set_process_id(&mut self, process_id: u32) {
		let x = process_id;
		match self {
			Self::Shared { process_id, .. } => *process_id = x,
			Self::Exclusive { process_id, .. } => *process_id = x,
		}
	}

	pub(crate) fn parse(raw: fuse_kernel::fuse_file_lock) -> Option<Self> {
		let range = LockRange::parse(raw);
		let process_id = raw.pid;
		match raw.r#type {
			F_RDLCK => Some(Self::Shared { range, process_id }),
			F_WRLCK => Some(Self::Exclusive { range, process_id }),
			_ => None,
		}
	}
}
