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

// GetlkRequest {{{

/// Request type for `FUSE_GETLK`.
pub struct GetlkRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_lk_in,
	lock_range: crate::LockRange,
}

impl GetlkRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[inline]
	#[must_use]
	pub fn owner(&self) -> crate::LockOwner {
		crate::LockOwner(self.body.owner)
	}

	#[inline]
	#[must_use]
	pub fn lock_mode(&self) -> crate::LockMode {
		crate::LockMode(self.body.lk.r#type)
	}

	#[inline]
	#[must_use]
	pub fn lock_range(&self) -> crate::LockRange {
		self.lock_range
	}
}

try_from_fuse_request!(GetlkRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_GETLK)?;

	let header = dec.header();
	decode::node_id(header.nodeid)?;

	let body: &kernel::fuse_lk_in = dec.next_sized()?;
	let lock_range = crate::LockRange::decode(&body.lk)?;
	Ok(Self { header, body, lock_range })
});

impl fmt::Debug for GetlkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("owner", &self.owner())
			.field("lock_mode", &self.lock_mode())
			.field("lock_range", &self.lock_range())
			.finish()
	}
}

// }}}
