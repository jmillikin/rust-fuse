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

#[cfg(test)]
mod readlink_test;

// ReadlinkRequest {{{

pub struct ReadlinkRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
}

impl ReadlinkRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ReadlinkRequest<'a> {
	fn decode_request(dec: fuse_io::RequestDecoder<'a>) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_READLINK);
		Ok(Self { header })
	}
}

// }}}

// ReadlinkResponse {{{

pub struct ReadlinkResponse<'a> {
	name: &'a CStr,
}

impl<'a> ReadlinkResponse<'a> {
	pub fn new() -> Self {
		ReadlinkResponse {
			name: CStr::from_bytes_with_nul(b"\x00").unwrap(),
		}
	}

	pub fn set_name(&mut self, name: &'a CStr) {
		self.name = name;
	}
}

impl fmt::Debug for ReadlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkResponse")
			.field("name", &self.name)
			.finish()
	}
}

impl fuse_io::EncodeResponse for ReadlinkResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_bytes(self.name.to_bytes_with_nul())
	}
}

// }}}
