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

impl BmapRequest<'_> {
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

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for BmapRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		let header = buf.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_BMAP);
		let mut dec = decode::RequestDecoder::new(buf);
		let raw = dec.next_sized()?;
		Ok(Self { header, raw })
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
}

impl fmt::Debug for BmapResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("BmapResponse")
			.field("block", &self.raw.block)
			.finish()
	}
}

impl encode::EncodeReply for BmapResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		_version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		enc.encode_sized(&self.raw)
	}
}

// }}}
