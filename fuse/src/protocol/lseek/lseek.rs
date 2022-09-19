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

#[cfg(rust_fuse_test = "lseek_test")]
mod lseek_test;

// LseekRequest {{{

pub struct LseekRequest<'a> {
	raw: &'a fuse_kernel::fuse_lseek_in,
	node_id: NodeId,
}

impl<'a> LseekRequest<'a> {
	pub fn from_fuse_request(
		request: &FuseRequest<'a>,
	) -> Result<Self, RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LSEEK)?;
		let raw = dec.next_sized()?;
		Ok(Self {
			raw,
			node_id: try_node_id(dec.header().nodeid)?,
		})
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	pub fn whence(&self) -> LseekWhence {
		LseekWhence(self.raw.whence)
	}
}

impl fmt::Debug for LseekRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LseekRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("offset", &self.raw.offset)
			.field("whence", &LseekWhence(self.raw.whence))
			.finish()
	}
}

#[derive(Eq, PartialEq)]
pub struct LseekWhence(u32);

impl LseekWhence {
	pub const SEEK_DATA: LseekWhence = LseekWhence(3);
	pub const SEEK_HOLE: LseekWhence = LseekWhence(4);
}

impl fmt::Debug for LseekWhence {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self.0 {
			3 => fmt.write_str("SEEK_DATA"),
			4 => fmt.write_str("SEEK_HOLE"),
			_ => self.0.fmt(fmt),
		}
	}
}

// }}}

// LseekResponse {{{

pub struct LseekResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_lseek_out,
}

impl<'a> LseekResponse<'a> {
	pub fn new() -> LseekResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_lseek_out { offset: 0 },
		}
	}

	pub fn set_offset(&mut self, offset: u64) {
		self.raw.offset = offset;
	}
}

impl fmt::Debug for LseekResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LseekResponse")
			.field("offset", &self.raw.offset)
			.finish()
	}
}

impl encode::EncodeReply for LseekResponse<'_> {
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
