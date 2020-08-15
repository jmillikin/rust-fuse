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

#[cfg(test)]
mod link_test;

// LinkRequest {{{

#[derive(Debug)]
pub struct LinkRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	name: &'a CStr,
	old_node_id: NodeId,
}

impl LinkRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn name(&self) -> &CStr {
		self.name
	}

	// TODO: rename to "source_node_id"
	pub fn old_node_id(&self) -> NodeId {
		self.old_node_id
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for LinkRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_LINK);

		let raw: &fuse_kernel::fuse_link_in = dec.next_sized()?;
		let name = dec.next_cstr()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
			name,
			old_node_id: try_node_id(raw.oldnodeid)?,
		})
	}
}

// }}}

// LinkResponse {{{

pub struct LinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> LinkResponse<'a> {
	pub fn new() -> LinkResponse<'a> {
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

impl fmt::Debug for LinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LinkResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl fuse_io::EncodeResponse for LinkResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		self.node().encode_entry(enc)
	}
}

// }}}
