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

//! Implements the `FUSE_FLUSH` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// FlushRequest {{{

/// Request type for `FUSE_FLUSH`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FLUSH` operation.
pub struct FlushRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	handle: u64,
	lock_owner: u64,
}

impl<'a> FlushRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		decode_request(request.buf, false)
	}

	pub fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		decode_request(request.buf, true)
	}

	#[must_use]
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.handle
	}

	#[must_use]
	pub fn lock_owner(&self) -> u64 {
		self.lock_owner
	}
}

impl fmt::Debug for FlushRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FlushRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.finish()
	}
}

fn decode_request<'a>(
	buf: decode::RequestBuf<'a>,
	is_cuse: bool,
) -> Result<FlushRequest<'a>, io::RequestError> {
	buf.expect_opcode(fuse_kernel::FUSE_FLUSH)?;

	let node_id = if is_cuse {
		crate::ROOT_ID
	} else {
		decode::node_id(buf.header().nodeid)?
	};
	let mut dec = decode::RequestDecoder::new(buf);

	let raw: &fuse_kernel::fuse_flush_in = dec.next_sized()?;
	Ok(FlushRequest {
		phantom: PhantomData,
		node_id,
		handle: raw.fh,
		lock_owner: raw.lock_owner,
	})
}

// }}}

// FlushResponse {{{

/// Response type for `FUSE_FLUSH`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FLUSH` operation.
pub struct FlushResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FlushResponse<'a> {
	#[must_use]
	pub fn new() -> FlushResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(FlushResponse<'_>);

impl fmt::Debug for FlushResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FlushResponse").finish()
	}
}

impl FlushResponse<'_> {
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
