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

#[cfg(rust_fuse_test = "removexattr_test")]
mod removexattr_test;

// RemovexattrRequest {{{

pub struct RemovexattrRequest<'a> {
	node_id: NodeId,
	name: &'a XattrName,
}

impl RemovexattrRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn name(&self) -> &XattrName {
		self.name
	}
}

impl fmt::Debug for RemovexattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrRequest")
			.field("node_id", &self.node_id)
			.field("name", &self.name)
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for RemovexattrRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		buf.expect_opcode(fuse_kernel::FUSE_REMOVEXATTR)?;

		let mut dec = decode::RequestDecoder::new(buf);
		let name = XattrName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			node_id: try_node_id(buf.header().nodeid)?,
			name,
		})
	}
}

// }}}

// RemovexattrResponse {{{

pub struct RemovexattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> RemovexattrResponse<'a> {
	pub fn new() -> RemovexattrResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for RemovexattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrResponse").finish()
	}
}

impl encode::EncodeReply for RemovexattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		_version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		enc.encode_header_only()
	}
}

// }}}
