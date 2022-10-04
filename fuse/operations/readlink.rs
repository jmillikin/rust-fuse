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

//! Implements the `FUSE_READLINK` operation.

// use core::ffi::CStr;
#[cfg(feature = "std")]
use std::ffi::CStr;

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugBytesAsString;

// ReadlinkRequest {{{

/// Request type for `FUSE_READLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READLINK` operation.
pub struct ReadlinkRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
}

impl ReadlinkRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
}

request_try_from! { ReadlinkRequest : fuse }

impl decode::Sealed for ReadlinkRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for ReadlinkRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_READLINK)?;
		Ok(Self {
			phantom: PhantomData,
			node_id: decode::node_id(dec.header().nodeid)?,
		})
	}
}

impl fmt::Debug for ReadlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkRequest")
			.field("node_id", &self.node_id)
			.finish()
	}
}

// }}}

// ReadlinkResponse {{{

/// Response type for `FUSE_READLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READLINK` operation.
pub struct ReadlinkResponse<'a> {
	target: &'a [u8],
}

impl<'a> ReadlinkResponse<'a> {
	#[cfg(feature = "std")]
	#[must_use]
	pub fn new(target: &'a CStr) -> ReadlinkResponse<'a> {
		Self { target: target.to_bytes() }
	}

	#[must_use]
	pub fn from_name(target: &'a NodeName) -> ReadlinkResponse<'a> {
		Self { target: target.as_bytes() }
	}
}

response_send_funcs!(ReadlinkResponse<'_>);

impl fmt::Debug for ReadlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkResponse")
			.field("target", &DebugBytesAsString(self.target))
			.finish()
	}
}

impl ReadlinkResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_bytes(self.target)
	}
}

// }}}
