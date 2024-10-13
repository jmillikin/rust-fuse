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

use crate::kernel;
use crate::server::decode;

// SetattrRequest {{{

/// Request type for `FUSE_SETATTR`.
#[derive(Clone, Copy)]
pub struct SetattrRequest<'a> {
	header: &'a kernel::fuse_in_header,
	raw: &'a kernel::fuse_setattr_in,
}

impl SetattrRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	fn get<T>(&self, bitmask: u32, value: T) -> Option<T> {
		if self.raw.valid & bitmask == 0 {
			return None;
		}
		Some(value)
	}

	#[must_use]
	fn get_timestamp(
		&self,
		bitmask: u32,
		seconds: u64,
		nanos: u32,
	) -> Option<crate::UnixTime> {
		if self.raw.valid & bitmask == 0 {
			return None;
		}
		Some(unsafe {
			crate::UnixTime::from_timespec_unchecked(seconds, nanos)
		})
	}

	#[must_use]
	pub fn handle(&self) -> Option<u64> {
		self.get(kernel::FATTR_FH, self.raw.fh)
	}

	#[must_use]
	pub fn size(&self) -> Option<u64> {
		self.get(kernel::FATTR_SIZE, self.raw.size)
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<crate::LockOwner> {
		self.get(
			kernel::FATTR_LOCKOWNER,
			crate::LockOwner(self.raw.lock_owner),
		)
	}

	#[must_use]
	pub fn atime(&self) -> Option<crate::UnixTime> {
		self.get_timestamp(
			kernel::FATTR_ATIME,
			self.raw.atime,
			self.raw.atimensec,
		)
	}

	#[must_use]
	pub fn atime_now(&self) -> bool {
		self.raw.valid & kernel::FATTR_ATIME_NOW > 0
	}

	#[must_use]
	pub fn mtime(&self) -> Option<crate::UnixTime> {
		self.get_timestamp(
			kernel::FATTR_MTIME,
			self.raw.mtime,
			self.raw.mtimensec,
		)
	}

	#[must_use]
	pub fn mtime_now(&self) -> bool {
		self.raw.valid & kernel::FATTR_MTIME_NOW > 0
	}

	#[must_use]
	pub fn ctime(&self) -> Option<crate::UnixTime> {
		self.get_timestamp(
			kernel::FATTR_CTIME,
			self.raw.ctime,
			self.raw.ctimensec,
		)
	}

	#[must_use]
	pub fn mode(&self) -> Option<crate::FileMode> {
		self.get(kernel::FATTR_MODE, crate::FileMode::new(self.raw.mode))
	}

	#[must_use]
	pub fn user_id(&self) -> Option<u32> {
		self.get(kernel::FATTR_UID, self.raw.uid)
	}

	#[must_use]
	pub fn group_id(&self) -> Option<u32> {
		self.get(kernel::FATTR_GID, self.raw.gid)
	}
}

try_from_fuse_request!(SetattrRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_SETATTR)?;
	let header = dec.header();
	decode::node_id(header.nodeid)?;
	let raw: &kernel::fuse_setattr_in = dec.next_sized()?;

	if raw.valid & kernel::FATTR_ATIME > 0 {
		decode::check_timespec_nanos(raw.atimensec)?;
	}
	if raw.valid & kernel::FATTR_MTIME > 0 {
		decode::check_timespec_nanos(raw.mtimensec)?;
	}
	if raw.valid & kernel::FATTR_CTIME > 0 {
		decode::check_timespec_nanos(raw.ctimensec)?;
	}

	Ok(Self { header, raw })
});

impl fmt::Debug for SetattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetattrRequest")
			.field("node_id", &self.node_id())
			.field("handle", &format_args!("{:?}", self.handle()))
			.field("size", &format_args!("{:?}", self.size()))
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("atime", &format_args!("{:?}", self.atime()))
			.field("atime_now", &self.atime_now())
			.field("mtime", &format_args!("{:?}", self.mtime()))
			.field("mtime_now", &self.mtime_now())
			.field("ctime", &format_args!("{:?}", self.ctime()))
			.field("mode", &format_args!("{:?}", self.mode()))
			.field("user_id", &format_args!("{:?}", self.user_id()))
			.field("group_id", &format_args!("{:?}", self.group_id()))
			.finish()
	}
}

// }}}
