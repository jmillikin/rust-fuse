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

use crate::internal::fuse_kernel;
use crate::lock;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// FlushRequest {{{

/// Request type for `FUSE_FLUSH`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FLUSH` operation.
pub struct FlushRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: node::Id,
	handle: u64,
	lock_owner: lock::Owner,
}

impl FlushRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		self.node_id
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.handle
	}

	#[must_use]
	pub fn lock_owner(&self) -> lock::Owner {
		self.lock_owner
	}
}

request_try_from! { FlushRequest : cuse fuse }

impl decode::Sealed for FlushRequest<'_> {}

impl<'a> decode::CuseRequest<'a> for FlushRequest<'a> {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, true)
	}
}

impl<'a> decode::FuseRequest<'a> for FlushRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, false)
	}
}

impl<'a> FlushRequest<'a> {
	fn decode_request(
		buf: decode::RequestBuf<'a>,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		buf.expect_opcode(fuse_kernel::FUSE_FLUSH)?;

		let node_id = if is_cuse {
			node::Id::ROOT
		} else {
			decode::node_id(buf.header().nodeid)?
		};
		let mut dec = decode::RequestDecoder::new(buf);

		let raw: &fuse_kernel::fuse_flush_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id,
			handle: raw.fh,
			lock_owner: lock::Owner::new(raw.lock_owner),
		})
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
