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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "symlink_test")]
mod symlink_test;

// SymlinkRequest {{{

/// Request type for [`FuseHandlers::symlink`].
///
/// [`FuseHandlers::symlink`]: ../../trait.FuseHandlers.html#method.symlink
pub struct SymlinkRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
	content: &'a [u8],
}

impl SymlinkRequest<'_> {
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

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for SymlinkRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::RequestError> {
		buf.expect_opcode(fuse_kernel::FUSE_SYMLINK)?;

		let mut dec = decode::RequestDecoder::new(buf);
		let content = dec.next_nul_terminated_bytes()?.to_bytes_without_nul();
		let name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			parent_id: try_node_id(buf.header().nodeid)?,
			name,
			content,
		})
	}
}

// }}}

// SymlinkResponse {{{

/// Response type for [`FuseHandlers::symlink`].
///
/// [`FuseHandlers::symlink`]: ../../trait.FuseHandlers.html#method.symlink
pub struct SymlinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> SymlinkResponse<'a> {
	pub fn new() -> SymlinkResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: Default::default(),
		}
	}

	pub fn node(&self) -> &Node {
		Node::new_ref(&self.raw)
	}

	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.raw)
	}
}

impl fmt::Debug for SymlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl encode::EncodeReply for SymlinkResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		self.node().encode_entry(enc, version_minor)
	}
}

// }}}
