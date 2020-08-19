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

impl RmdirRequest<'_> {
	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	pub fn name(&self) -> &NodeName {
		self.name
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for RmdirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_RMDIR);

		Ok(Self {
			parent_id: try_node_id(header.nodeid)?,
			name: NodeName::new(dec.next_nul_terminated_bytes()?),
		})
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

impl fuse_io::EncodeResponse for RmdirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_header_only()
	}
}

// }}}
