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

//! Implements the `FUSE_SYMLINK` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::Node;
use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugBytesAsString;

// SymlinkRequest {{{

/// Request type for `FUSE_SYMLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYMLINK` operation.
pub struct SymlinkRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
	content: &'a [u8],
}

impl<'a> SymlinkRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_SYMLINK)?;
		let content = dec.next_nul_terminated_bytes()?.to_bytes_without_nul();
		let name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name,
			content,
		})
	}

	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	pub fn name(&self) -> &NodeName {
		self.name
	}

	pub fn content(&self) -> &[u8] {
		self.content
	}
}

impl fmt::Debug for SymlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkRequest")
			.field("parent_id", &self.parent_id)
			.field("name", &self.name)
			.field("content", &DebugBytesAsString(self.content))
			.finish()
	}
}

// }}}

// SymlinkResponse {{{

/// Response type for `FUSE_SYMLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYMLINK` operation.
pub struct SymlinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> SymlinkResponse<'a> {
	pub fn new() -> SymlinkResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_entry_out::zeroed(),
		}
	}

	pub fn node(&self) -> &Node {
		Node::new_ref(&self.raw)
	}

	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.raw)
	}
}

response_send_funcs!(SymlinkResponse<'_>);

impl fmt::Debug for SymlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl SymlinkResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		self.node().encode_entry(enc, ctx.version_minor)
	}
}

// }}}
