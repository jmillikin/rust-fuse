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

// SetxattrRequest {{{

pub struct SetxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_setxattr_in,
	name: &'a CStr,
	value: &'a [u8],
}

impl SetxattrRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn flags(&self) -> u32 {
		self.raw.flags
	}

	pub fn name(&self) -> &CStr {
		self.name
	}

	pub fn value(&self) -> &[u8] {
		self.value
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for SetxattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_SETXATTR);

		let raw: &'a fuse_kernel::fuse_setxattr_in = dec.next_sized()?;
		let name = dec.next_cstr()?;
		let value = dec.next_bytes(raw.size)?;
		Ok(Self {
			header,
			raw,
			name,
			value,
		})
	}
}

// }}}

// SetxattrResponse {{{

pub struct SetxattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SetxattrResponse<'a> {
	pub fn new() -> SetxattrResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for SetxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetxattrResponse").finish()
	}
}

impl fuse_io::EncodeResponse for SetxattrResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_header_only()
	}
}

// }}}
