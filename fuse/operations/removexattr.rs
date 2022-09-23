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

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::XattrName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// RemovexattrRequest {{{

pub struct RemovexattrRequest<'a> {
	node_id: NodeId,
	name: &'a XattrName,
}

impl<'a> RemovexattrRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_REMOVEXATTR)?;
		let name = XattrName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			node_id: decode::node_id(dec.header().nodeid)?,
			name,
		})
	}

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

	response_send_funcs!();
}

impl fmt::Debug for RemovexattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrResponse").finish()
	}
}

impl RemovexattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_header_only()
	}
}

// }}}
