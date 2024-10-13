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

use core::fmt;

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

	/// Creates a new `UnixTime` with the given offset from the epoch.
	///
	/// Returns `None` if the nanoseconds value exceeds 999,999,999.
	#[inline]
	#[must_use]
	pub const fn from_seconds(seconds: i64) -> UnixTime {
		Self { seconds, nanos: 0 }
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

impl fmt::Debug for UnixTime {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_tuple("UnixTime")
			.field(&format_args!("{:?}.{:09?}", self.seconds, self.nanos))
			.finish()
	}
}
