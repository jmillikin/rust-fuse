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

// LseekRequest {{{

pub struct LseekRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_lseek_in,
}

impl LseekRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	// TODO: SEEK_HOLE, SEEK_DATA
	//
	// maybe SEEK_SET, SEEK_CUR, SEEK_END ? does the kernel ever pass these
	// through to the FS?

	pub fn whence(&self) -> u32 {
		self.raw.whence
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for LseekRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_LSEEK);
		let raw = dec.next_sized()?;
		Ok(Self { header, raw })
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

impl fuse_io::EncodeResponse for LseekResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Error> {
		enc.encode_sized(&self.raw)
	}
}

// }}}
