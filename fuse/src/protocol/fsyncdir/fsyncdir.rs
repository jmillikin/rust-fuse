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

// FsyncdirRequest {{{

pub struct FsyncdirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	fh: u64,
	fsync_flags: u32,
}

impl FsyncdirRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.fh
	}

	pub fn datasync(&self) -> bool {
		(self.fsync_flags & 0x1) > 0
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for FsyncdirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_FSYNCDIR);

		let raw: &fuse_kernel::fuse_fsync_in = dec.next_sized()?;
		Ok(Self {
			header,
			fh: raw.fh,
			fsync_flags: raw.fsync_flags,
		})
	}
}

// }}}

// FsyncdirResponse {{{

pub struct FsyncdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FsyncdirResponse<'a> {
	pub fn new() -> FsyncdirResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for FsyncdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirResponse").finish()
	}
}

impl fuse_io::EncodeResponse for FsyncdirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_header_only()
	}
}

// }}}
