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
use crate::server;
use crate::server::decode;

// ReleaseRequest {{{

/// Request type for `FUSE_RELEASE`.
#[derive(Clone, Copy)]
pub struct ReleaseRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_release_in<'a>>,
}

impl ReleaseRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		crate::NodeId::new(self.header.nodeid).unwrap_or(crate::NodeId::ROOT)
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

try_from_cuse_request!(ReleaseRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	Self::try_from(request.inner, version_minor, true)
});

try_from_fuse_request!(ReleaseRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	Self::try_from(request.inner, version_minor, false)
});

impl<'a> ReleaseRequest<'a> {
	fn try_from(
		request: server::Request<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_RELEASE)?;

		let header = dec.header();
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let body = if version_minor >= 8 {
			let body_v7p8 = dec.next_sized()?;
			compat::Versioned::new_release_v7p8(version_minor, body_v7p8)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_release_v7p1(version_minor, body_v7p1)
		};

		Ok(Self { header, body })
	}
}

impl fmt::Debug for ReleaseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}
