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

//! Implements the `FUSE_LOOKUP` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::Node;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// LookupRequest {{{

/// Request type for `FUSE_LOOKUP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LOOKUP` operation.
#[derive(Debug)]
pub struct LookupRequest<'a> {
	parent_id: node::Id,
	name: &'a node::Name,
}

impl LookupRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}
}

request_try_from! { LookupRequest : fuse }

impl decode::Sealed for LookupRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for LookupRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LOOKUP)?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name: dec.next_node_name()?,
		})
	}
}

// }}}

// LookupResponse {{{

/// Response type for `FUSE_LOOKUP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LOOKUP` operation.
pub struct LookupResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry_out: fuse_kernel::fuse_entry_out,
}

impl<'a> LookupResponse<'a> {
	#[must_use]
	pub fn new() -> LookupResponse<'a> {
		Self {
			phantom: PhantomData,
			entry_out: fuse_kernel::fuse_entry_out::zeroed(),
		}
	}

	#[must_use]
	pub fn node(&self) -> &Node {
		Node::new_ref(&self.entry_out)
	}

	#[must_use]
	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.entry_out)
	}
}

response_send_funcs!(LookupResponse<'_>);

impl fmt::Debug for LookupResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LookupResponse")
			.field("node", self.node())
			.finish()
	}
}

impl LookupResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		// In early versions of FUSE, `fuse_entry_out::nodeid` was a required
		// field and must be non-zero. FUSE v7.4 relaxed this so that a zero
		// node ID was the same as returning ENOENT, but with a cache hint.
		if self.entry_out.nodeid == 0 && ctx.version_minor < 4 {
			return enc.encode_error(crate::Error::NOT_FOUND);
		}
		self.node().encode_entry(enc, ctx.version_minor)
	}
}

// }}}
