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
use crate::kernel;
use crate::server::decode;

// GetattrRequest {{{

/// Request type for `FUSE_GETATTR`.
pub struct GetattrRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_getattr_in<'a>>,
}

impl GetattrRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> Option<u64> {
		let body = self.body.as_v7p9()?;
		if (body.getattr_flags & kernel::FUSE_GETATTR_FH) > 0 {
			return Some(body.fh);
		}
		None
	}
}

try_from_fuse_request!(GetattrRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_GETATTR)?;

	let header = dec.header();
	decode::node_id(header.nodeid)?;

	let body = if version_minor >= 9 {
		let body_v7p9 = dec.next_sized()?;
		compat::Versioned::new_getattr_v7p9(version_minor, body_v7p9)
	} else {
		compat::Versioned::new_getattr_v7p1(version_minor)
	};

	Ok(Self { header, body })
});

impl fmt::Debug for GetattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetattrRequest")
			.field("node_id", &self.node_id())
			.field("handle", &format_args!("{:?}", &self.handle()))
			.finish()
	}
}

// }}}
