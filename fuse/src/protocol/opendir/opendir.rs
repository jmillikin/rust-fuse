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

use crate::protocol::node;
use crate::protocol::prelude::*;

#[cfg(test)]
mod opendir_test;

// OpendirRequest {{{

/// **\[UNSTABLE\]**
pub struct OpendirRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: node::NodeId,
	flags: u32,
}

impl OpendirRequest<'_> {
	pub fn node_id(&self) -> node::NodeId {
		self.node_id
	}

	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for OpendirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_OPENDIR);

		let raw: &'a fuse_kernel::fuse_open_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
			flags: raw.flags,
		})
	}
}

// }}}

// OpendirResponse {{{

/// **\[UNSTABLE\]**
pub struct OpendirResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_open_out,
}

impl OpendirResponse<'_> {
	pub fn new() -> Self {
		OpendirResponse {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_open_out {
				fh: 0,
				open_flags: 0,
				padding: 0,
			},
		}
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.raw.fh = handle;
	}

	pub fn flags(&self) -> u32 {
		self.raw.open_flags
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.raw.open_flags = flags;
	}
}

impl fmt::Debug for OpendirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpendirResponse")
			.field("handle", &self.raw.fh)
			.field("flags", &self.raw.open_flags)
			.finish()
	}
}

impl fuse_io::EncodeResponse for OpendirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_sized(&self.raw)
	}
}

// }}}
