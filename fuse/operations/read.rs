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

// ReadRequest {{{

/// Request type for `FUSE_READ`.
pub struct ReadRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_read_in<'a>>,
}

impl ReadRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		crate::NodeId::new(self.header.nodeid).unwrap_or(crate::NodeId::ROOT)
	}

	#[must_use]
	pub fn size(&self) -> u32 {
		self.body.as_v7p1().size
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.body.as_v7p1().offset
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
		let body = self.body.as_v7p9()?;
		if body.read_flags & kernel::FUSE_READ_LOCKOWNER == 0 {
			return None;
		}
		Some(crate::LockOwner(body.lock_owner))
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		if let Some(body) = self.body.as_v7p9() {
			return body.flags;
		}
		0
	}
}

try_from_cuse_request!(ReadRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	Self::try_from(request.inner, version_minor, true)
});

try_from_fuse_request!(ReadRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	Self::try_from(request.inner, version_minor, false)
});

impl<'a> ReadRequest<'a> {
	fn try_from(
		request: server::Request<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<ReadRequest<'a>, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_READ)?;
		let header = dec.header();

		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let body = if version_minor >= 9 {
			let body_v7p9 = dec.next_sized()?;
			compat::Versioned::new_read_v7p9(version_minor, body_v7p9)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_read_v7p1(version_minor, body_v7p1)
		};

		Ok(ReadRequest { header, body })
	}
}

impl fmt::Debug for ReadRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadRequest")
			.field("node_id", &self.node_id())
			.field("size", &self.size())
			.field("offset", &self.offset())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", &self.lock_owner()))
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}
