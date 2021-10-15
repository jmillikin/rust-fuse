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

#[cfg(rust_fuse_test = "fallocate_test")]
mod fallocate_test;

// FallocateRequest {{{

const FALLOC_FL_KEEP_SIZE: u32 = 1 << 0;
const FALLOC_FL_PUNCH_HOLE: u32 = 1 << 1;
const FALLOC_FL_COLLAPSE_RANGE: u32 = 1 << 3;
const FALLOC_FL_ZERO_RANGE: u32 = 1 << 4;
const FALLOC_FL_INSERT_RANGE: u32 = 1 << 5;
const FALLOC_FL_UNSHARE_RANGE: u32 = 1 << 6;

pub struct FallocateRequest<'a> {
	raw: &'a fuse_kernel::fuse_fallocate_in,
	node_id: NodeId,
	mode: FallocateMode,
}

impl FallocateRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	pub fn length(&self) -> u64 {
		self.raw.length
	}

	pub fn mode(&self) -> FallocateMode {
		self.mode
	}
}

bitflags_struct! {
	/// Mode bits set in an [`FallocateRequest`].
	///
	/// [`FallocateRequest`]: struct.FallocateRequest.html
	pub struct FallocateMode(u32);

	FALLOC_FL_KEEP_SIZE: keep_size,
	FALLOC_FL_PUNCH_HOLE: punch_hole,
	FALLOC_FL_COLLAPSE_RANGE: collapse_range,
	FALLOC_FL_ZERO_RANGE: zero_range,
	FALLOC_FL_INSERT_RANGE: insert_range,
	FALLOC_FL_UNSHARE_RANGE: unshare_range,
}

impl fmt::Debug for FallocateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("offset", &self.raw.offset)
			.field("length", &self.raw.length)
			.field("mode", &self.mode)
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for FallocateRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::RequestError> {
		buf.expect_opcode(fuse_kernel::FUSE_FALLOCATE)?;

		let mut dec = decode::RequestDecoder::new(buf);
		let raw = dec.next_sized()?;
		Ok(Self {
			raw,
			node_id: try_node_id(buf.header().nodeid)?,
			mode: FallocateMode::from_bits(raw.mode),
		})
	}
}

// }}}

// FallocateResponse {{{

pub struct FallocateResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FallocateResponse<'a> {
	pub fn new() -> FallocateResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for FallocateResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateResponse").finish()
	}
}

impl encode::EncodeReply for FallocateResponse<'_> {
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
