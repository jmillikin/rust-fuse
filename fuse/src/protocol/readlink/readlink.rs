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

#[cfg(rust_fuse_test = "readlink_test")]
mod readlink_test;

// ReadlinkRequest {{{

/// Request type for [`FuseHandlers::readlink`].
///
/// [`FuseHandlers::readlink`]: ../../trait.FuseHandlers.html#method.readlink
pub struct ReadlinkRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
}

impl ReadlinkRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
}

impl fmt::Debug for ReadlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkRequest")
			.field("node_id", &self.node_id)
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for ReadlinkRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		let header = buf.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_READLINK);
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
		})
	}
}

// }}}

// ReadlinkResponse {{{

/// Response type for [`FuseHandlers::readlink`].
///
/// [`FuseHandlers::readlink`]: ../../trait.FuseHandlers.html#method.readlink
pub struct ReadlinkResponse<'a> {
	name: &'a NodeName,
}

impl<'a> ReadlinkResponse<'a> {
	pub fn from_name(name: &'a NodeName) -> ReadlinkResponse<'a> {
		Self { name }
	}
}

impl fmt::Debug for ReadlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkResponse")
			.field("name", &self.name)
			.finish()
	}
}

impl encode::EncodeReply for ReadlinkResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		_version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		enc.encode_bytes(self.name.as_bytes())
	}
}

// }}}
