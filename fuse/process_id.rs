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
use core::num::NonZeroU32;

/// Represents a process that initiated a FUSE request.
///
/// The concept of a "process ID" is not fully specified by POSIX, and some
/// platforms may report process IDs that don't match the intuitive userland
/// meaning. For example, platforms that represent processes as a group of
/// threads might populate a request's process ID from the thread ID (TID)
/// rather than the thread group ID (TGID).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProcessId {
	pid: NonZeroU32,
}

impl ProcessId {
	/// Creates a new `ProcessId` if the given PID is not zero.
	#[inline]
	#[must_use]
	pub fn new(pid: u32) -> Option<ProcessId> {
		Some(Self {
			pid: NonZeroU32::new(pid)?,
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
