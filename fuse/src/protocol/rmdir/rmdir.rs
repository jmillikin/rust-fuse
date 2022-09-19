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

#[cfg(rust_fuse_test = "rmdir_test")]
mod rmdir_test;

// RmdirRequest {{{

/// Request type for [`FuseHandlers::rmdir`].
///
/// [`FuseHandlers::rmdir`]: ../../trait.FuseHandlers.html#method.rmdir
#[derive(Debug)]
pub struct RmdirRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
}

impl<'a> RmdirRequest<'a> {
	pub fn from_fuse_request(
		request: &FuseRequest<'a>,
	) -> Result<Self, RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_RMDIR)?;
		Ok(Self {
			parent_id: try_node_id(dec.header().nodeid)?,
			name: NodeName::new(dec.next_nul_terminated_bytes()?),
		})
	}

	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	pub fn name(&self) -> &NodeName {
		self.name
	}
}

// }}}

// RmdirResponse {{{

/// Response type for [`FuseHandlers::rmdir`].
///
/// [`FuseHandlers::rmdir`]: ../../trait.FuseHandlers.html#method.rmdir
pub struct RmdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> RmdirResponse<'a> {
	pub fn new() -> RmdirResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for RmdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RmdirResponse").finish()
	}
}

impl encode::EncodeReply for RmdirResponse<'_> {
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
