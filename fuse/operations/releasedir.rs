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

use crate::internal::compat;
use crate::internal::debug;
use crate::kernel;
use crate::server::decode;

// ReleasedirRequest {{{

/// Request type for `FUSE_RELEASEDIR`.
pub struct ReleasedirRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_release_in<'a>>,
}

impl ReleasedirRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	/// The value set in [`fuse_open_out::fh`], or zero if not set.
	///
	/// [`fuse_open_out::fh`]: crate::kernel::fuse_open_out::fh
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.as_v7p1().fh
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<crate::LockOwner> {
		let body = self.body.as_v7p8()?;
		if body.release_flags & kernel::FUSE_RELEASE_FLOCK_UNLOCK == 0 {
			return None;
		}
		Some(crate::LockOwner(body.lock_owner))
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.as_v7p1().flags
	}
}

try_from_fuse_request!(ReleasedirRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_RELEASEDIR)?;

	let header = dec.header();
	decode::node_id(header.nodeid)?;

	let body = if version_minor >= 8 {
		let body_v7p8 = dec.next_sized()?;
		compat::Versioned::new_release_v7p8(version_minor, body_v7p8)
	} else {
		let body_v7p1 = dec.next_sized()?;
		compat::Versioned::new_release_v7p1(version_minor, body_v7p1)
	};

	Ok(Self { header, body })
});

impl fmt::Debug for ReleasedirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleasedirRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}
