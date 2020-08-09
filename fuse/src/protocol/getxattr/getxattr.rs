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
mod getxattr_test;

// GetxattrRequest {{{

/// **\[UNSTABLE\]**
pub struct GetxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_getxattr_in,
	name: &'a CStr,
}

impl GetxattrRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn size(&self) -> Option<u32> {
		if self.raw.size == 0 {
			None
		} else {
			Some(self.raw.size)
		}
	}

	pub fn name(&self) -> &CStr {
		self.name
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for GetxattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_GETXATTR);

		let raw = dec.next_sized()?;
		let name = dec.next_cstr()?;
		Ok(Self { header, raw, name })
	}
}

// }}}

// GetxattrResponse {{{

/// **\[UNSTABLE\]**
pub struct GetxattrResponse<'a> {
	request_size: u32,
	raw: fuse_kernel::fuse_getxattr_out,
	buf: &'a [u8],
}

impl<'a> GetxattrResponse<'a> {
	// TODO: this API is bad. user should construct with new() and then
	// set the value to either a &[u8] or a Vec<u8>, via a Cow<>.
	pub fn for_request(request: &GetxattrRequest) -> Self {
		// Clamp the maximum response size to avoid caring about u32 overflow.
		// This limit is far higher than existing kernel implementations support.
		GetxattrResponse {
			request_size: cmp::min(request.raw.size, 1 << 30),
			raw: Default::default(),
			buf: &[],
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.raw.size = size;
	}

	pub fn set_value(&mut self, value: &'a [u8]) -> io::Result<()> {
		if value.len() > u32::MAX as usize {
			return Err(io::Error::from_raw_os_error(
				errors::ERANGE.get() as i32
			));
		}
		let value_len = value.len() as u32;

		if self.request_size == 0 {
			self.raw.size = value_len;
			return Ok(());
		}

		if value_len > self.request_size {
			return Err(io::Error::from_raw_os_error(
				errors::ERANGE.get() as i32
			));
		}
		self.raw.size = 0;
		self.buf = value;
		Ok(())
	}
}

impl fmt::Debug for GetxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetxattrResponse").finish()
	}
}

impl fuse_io::EncodeResponse for GetxattrResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		if self.raw.size != 0 {
			enc.encode_sized(&self.raw)
		} else {
			enc.encode_bytes(&self.buf)
		}
	}
}

// }}}
