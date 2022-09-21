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

// BmapRequest {{{

pub struct BmapRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_bmap_in,
}

impl<'a> BmapRequest<'a> {
	pub fn from_fuse_request(
		request: &FuseRequest<'a>,
	) -> Result<Self, RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_BMAP)?;

		let header = dec.header();
		let raw = dec.next_sized()?;
		Ok(Self { header, raw })
	}

	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn block(&self) -> u64 {
		self.raw.block
	}

	pub fn blocksize(&self) -> u32 {
		self.raw.blocksize
	}
}

// }}}

// BmapResponse {{{

pub struct BmapResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_bmap_out,
}

impl<'a> BmapResponse<'a> {
	pub fn new() -> BmapResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: Default::default(),
		}
	}

	pub fn block(self) -> u64 {
		self.raw.block
	}

	pub fn set_block(&mut self, block: u64) {
		self.raw.block = block;
	}

	response_send_funcs!();
}

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
		ctx: &crate::server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&self.raw)
	}
}

// }}}
