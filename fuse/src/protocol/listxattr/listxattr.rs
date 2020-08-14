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
mod listxattr_test;

// ListxattrRequest {{{

pub struct ListxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	size: u32,
}

impl ListxattrRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}
	pub fn size(&self) -> Option<u32> {
		if self.size == 0 {
			None
		} else {
			Some(self.size)
		}
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ListxattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_LISTXATTR);

		let raw: &fuse_kernel::fuse_getxattr_in = dec.next_sized()?;
		Ok(Self {
			header,
			size: raw.size,
		})
	}
}

// }}}

// ListxattrResponse {{{

pub struct ListxattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
	request_size: u32,
	raw: fuse_kernel::fuse_getxattr_out,
	buf: Vec<u8>,
}

impl ListxattrResponse<'_> {
	// TODO: fix construction API
	pub fn new(request: &ListxattrRequest) -> Self {
		// Clamp the maximum response size to avoid caring about u32 overflow.
		// This limit is far higher than existing kernel implementations support.
		ListxattrResponse {
			phantom: PhantomData,
			request_size: cmp::min(request.size, 1 << 30),
			raw: Default::default(),
			buf: Vec::new(),
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.raw.size = size;
	}

	pub fn push<'a>(&mut self, name: &'a CStr) -> io::Result<()> {
		let bytes = name.to_bytes_with_nul();
		if bytes.len() > u32::MAX as usize {
			return Err(io::Error::from_raw_os_error(
				errors::ERANGE.get() as i32
			));
		}
		let bytes_len = bytes.len() as u32;

		let old_buf_size: u32;
		if self.request_size == 0 {
			old_buf_size = self.raw.size;
		} else {
			old_buf_size = self.buf.len() as u32;
		}

		let new_buf_size = match old_buf_size.checked_add(bytes_len) {
			Some(x) => Ok(x),
			None => {
				Err(io::Error::from_raw_os_error(errors::ERANGE.get() as i32))
			},
		}?;

		if self.request_size == 0 {
			self.raw.size = new_buf_size;
			return Ok(());
		}

		if new_buf_size > self.request_size {
			return Err(io::Error::from_raw_os_error(
				errors::ERANGE.get() as i32
			));
		}
		self.buf.extend_from_slice(bytes);
		Ok(())
	}
}

impl fmt::Debug for ListxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ListxattrResponse").finish()
	}
}

impl fuse_io::EncodeResponse for ListxattrResponse<'_> {
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
