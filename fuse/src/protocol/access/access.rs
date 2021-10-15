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

#[cfg(rust_fuse_test = "access_test")]
mod access_test;

// AccessRequest {{{

/// Request type for [`FuseHandlers::access`].
///
/// [`FuseHandlers::access`]: ../../trait.FuseHandlers.html#method.access
pub struct AccessRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	mask: u32,
}

impl AccessRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn mask(&self) -> u32 {
		self.mask
	}
}

impl fmt::Debug for AccessRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("AccessRequest")
			.field("node_id", &self.node_id)
			.field("mask", &self.mask)
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for AccessRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::RequestError> {
		buf.expect_opcode(fuse_kernel::FUSE_ACCESS)?;

		let mut dec = decode::RequestDecoder::new(buf);
		let raw: &'a fuse_kernel::fuse_access_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(buf.header().nodeid)?,
			mask: raw.mask,
		})
	}
}

// }}}

// AccessResponse {{{

/// Response type for [`FuseHandlers::access`].
///
/// [`FuseHandlers::access`]: ../../trait.FuseHandlers.html#method.access
pub struct AccessResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> AccessResponse<'a> {
	pub fn new() -> AccessResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for AccessResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("AccessResponse").finish()
	}
}

impl encode::EncodeReply for AccessResponse<'_> {
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
