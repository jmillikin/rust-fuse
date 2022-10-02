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

//! Implements the `FUSE_RMDIR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// RmdirRequest {{{

/// Request type for `FUSE_RMDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RMDIR` operation.
#[derive(Debug)]
pub struct RmdirRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
}

impl<'a> RmdirRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_RMDIR)?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name: NodeName::new(dec.next_nul_terminated_bytes()?),
		})
	}

	#[must_use]
	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &NodeName {
		self.name
	}
}

// }}}

// RmdirResponse {{{

/// Response type for `FUSE_RMDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RMDIR` operation.
pub struct RmdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> RmdirResponse<'a> {
	#[must_use]
	pub fn new() -> RmdirResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(RmdirResponse<'_>);

impl fmt::Debug for RmdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RmdirResponse").finish()
	}
}

impl RmdirResponse<'_> {
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
