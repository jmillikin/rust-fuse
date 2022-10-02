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

//! Implements the `FUSE_BMAP` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// BmapRequest {{{

/// Request type for `FUSE_BMAP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_BMAP` operation.
pub struct BmapRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_bmap_in,
}

impl<'a> BmapRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_BMAP)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}

	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn block(&self) -> u64 {
		self.body.block
	}

	#[must_use]
	pub fn block_size(&self) -> u32 {
		self.body.blocksize
	}
}

impl fmt::Debug for BmapRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("BmapRequest")
			.field("node_id", &self.node_id())
			.field("block", &self.block())
			.field("block_size", &self.block_size())
			.finish()
	}
}

// }}}

// BmapResponse {{{

/// Response type for `FUSE_BMAP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_BMAP` operation.
pub struct BmapResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_bmap_out,
}

impl<'a> BmapResponse<'a> {
	#[must_use]
	pub fn new() -> BmapResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_bmap_out::zeroed(),
		}
	}

	#[must_use]
	pub fn block(self) -> u64 {
		self.raw.block
	}

	pub fn set_block(&mut self, block: u64) {
		self.raw.block = block;
	}
}

response_send_funcs!(BmapResponse<'_>);

impl fmt::Debug for BmapResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("BmapResponse")
			.field("block", &self.raw.block)
			.finish()
	}
}

impl BmapResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&self.raw)
	}
}

// }}}
