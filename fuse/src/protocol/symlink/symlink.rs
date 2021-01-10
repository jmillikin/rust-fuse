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

impl<'a> fuse_io::DecodeRequest<'a> for SymlinkRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_SYMLINK);

		let content = dec.next_nul_terminated_bytes()?.to_bytes_without_nul();
		let name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			parent_id: try_node_id(header.nodeid)?,
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

impl fuse_io::EncodeResponse for SymlinkResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		self.node().encode_entry(enc)
	}
}

// }}}
