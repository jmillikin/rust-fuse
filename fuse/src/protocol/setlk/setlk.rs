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

// SetlkRequest {{{

pub struct SetlkRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_lk_in,
	block: bool,
}

impl SetlkRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn owner(&self) -> u64 {
		self.raw.owner
	}

	// TODO: the lock itself

	pub fn flock(&self) -> bool {
		(self.raw.lk_flags & fuse_kernel::FUSE_LK_FLOCK) > 0
	}

	pub fn block(&self) -> bool {
		self.block
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for SetlkRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		let block: bool;
		if header.opcode == fuse_kernel::FUSE_SETLKW {
			block = true;
		} else {
			debug_assert!(header.opcode == fuse_kernel::FUSE_SETLK);
			block = false;
		}
		let raw = dec.next_sized()?;
		Ok(Self { header, raw, block })
	}
}

// }}}

// SetlkResponse {{{

pub struct SetlkResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SetlkResponse<'a> {
	pub fn new() -> SetlkResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for SetlkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetlkResponse").finish()
	}
}

impl fuse_io::EncodeResponse for SetlkResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Error> {
		enc.encode_header_only()
	}
}

// }}}
