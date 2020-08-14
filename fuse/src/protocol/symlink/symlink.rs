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
mod symlink_test;

// SymlinkRequest {{{

pub struct SymlinkRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	old_name: &'a CStr,
	new_name: &'a CStr,
}

impl SymlinkRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn old_name(&self) -> &CStr {
		self.old_name
	}

	pub fn new_name(&self) -> &CStr {
		self.new_name
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for SymlinkRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_SYMLINK);

		let old_name = dec.next_cstr()?;
		let new_name = dec.next_cstr()?;
		Ok(Self {
			header,
			old_name,
			new_name,
		})
	}
}

// }}}

// SymlinkResponse {{{

pub struct SymlinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl SymlinkResponse<'_> {
	pub fn new() -> Self {
		SymlinkResponse {
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
	) -> std::io::Result<()> {
		self.node().encode_entry(enc)
	}
}

// }}}
