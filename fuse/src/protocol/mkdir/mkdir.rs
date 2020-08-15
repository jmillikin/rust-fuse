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
mod mkdir_test;

// MkdirRequest {{{

pub struct MkdirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	name: &'a CStr,
	mode: u32,
	umask: u32,
}

impl MkdirRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn name(&self) -> &CStr {
		self.name
	}

	pub fn mode(&self) -> u32 {
		self.mode
	}

	pub fn umask(&self) -> u32 {
		self.umask
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for MkdirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_MKDIR);

		let raw: &fuse_kernel::fuse_mkdir_in = dec.next_sized()?;
		let name = dec.next_cstr()?;
		Ok(Self {
			header,
			name,
			mode: raw.mode,
			umask: raw.umask,
		})
	}
}

// }}}

// MkdirResponse {{{

pub struct MkdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> MkdirResponse<'a> {
	pub fn new() -> MkdirResponse<'a> {
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

impl fmt::Debug for MkdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MkdirResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl fuse_io::EncodeResponse for MkdirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		self.node().encode_entry(enc)
	}
}

// }}}
