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
mod unlink_test;

// UnlinkRequest {{{

pub struct UnlinkRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	name: &'a CStr,
}

impl UnlinkRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn name(&self) -> &CStr {
		self.name
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for UnlinkRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_UNLINK);

		let name = dec.next_cstr()?;
		Ok(Self { header, name })
	}
}

// }}}

// UnlinkResponse {{{

pub struct UnlinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> UnlinkResponse<'a> {
	pub fn new() -> UnlinkResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for UnlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("UnlinkResponse").finish()
	}
}

impl fuse_io::EncodeResponse for UnlinkResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_header_only()
	}
}

// }}}
