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
mod read_test;

// ReadRequest {{{

pub struct ReadRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	handle: u64,
	offset: u64,
	size: u32,
	read_flags: u32,
	lock_owner: u64,
	flags: u32,
}

impl ReadRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn offset(&self) -> u64 {
		self.offset
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	pub fn lock_owner(&self) -> Option<u64> {
		if self.read_flags & fuse_kernel::FUSE_READ_LOCKOWNER == 0 {
			return None;
		}
		Some(self.lock_owner)
	}

	pub fn flags(&self) -> u32 {
		self.flags
	}
}

#[repr(C)]
pub(crate) struct fuse_read_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) offset: u64,
	pub(crate) size: u32,
	pub(crate) padding: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for ReadRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_READ);

		// FUSE v7.9 added new fields to `fuse_read_in`.
		if dec.version().minor() < 9 {
			let raw: &'a fuse_read_in_v7p1 = dec.next_sized()?;
			return Ok(Self {
				header,
				handle: raw.fh,
				offset: raw.offset,
				size: raw.size,
				read_flags: 0,
				lock_owner: 0,
				flags: 0,
			});
		}

		let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;
		Ok(Self {
			header,
			handle: raw.fh,
			offset: raw.offset,
			size: raw.size,
			read_flags: raw.read_flags,
			lock_owner: raw.lock_owner,
			flags: raw.flags,
		})
	}
}

// }}}

// ReadResponse {{{

pub struct ReadResponse<'a> {
	request_size: u32,
	buf: &'a [u8],
}

impl<'a> ReadResponse<'a> {
	// TODO: fix API
	pub fn new(request: &ReadRequest) -> Self {
		Self {
			request_size: request.size,
			buf: &[],
		}
	}

	pub fn set_value(&mut self, value: &'a [u8]) -> io::Result<()> {
		if value.len() > self.request_size as usize {
			return Err(io::Error::from_raw_os_error(libc::ERANGE));
		}
		self.buf = value;
		Ok(())
	}
}

impl fuse_io::EncodeResponse for ReadResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_bytes(self.buf)
	}
}

// }}}
