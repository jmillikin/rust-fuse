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

//! Implements the `FUSE_FALLOCATE` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugHexU32;

// FallocateRequest {{{

/// Request type for `FUSE_FALLOCATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FALLOCATE` operation.
pub struct FallocateRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_fallocate_in,
}

impl FallocateRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.body.offset
	}

	#[must_use]
	pub fn length(&self) -> u64 {
		self.body.length
	}

	#[must_use]
	pub fn fallocate_flags(&self) -> crate::FallocateFlags {
		self.body.mode
	}
}

request_try_from! { FallocateRequest : fuse }

impl decode::Sealed for FallocateRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for FallocateRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_FALLOCATE)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for FallocateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("offset", &self.offset())
			.field("length", &self.length())
			.field("fallocate_flags", &DebugHexU32(self.fallocate_flags()))
			.finish()
	}
}

// }}}

// FallocateResponse {{{

/// Response type for `FUSE_FALLOCATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FALLOCATE` operation.
pub struct FallocateResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FallocateResponse<'a> {
	#[must_use]
	pub fn new() -> FallocateResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(FallocateResponse<'_>);

impl fmt::Debug for FallocateResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateResponse").finish()
	}
}

impl FallocateResponse<'_> {
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
