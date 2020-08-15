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
mod write_test;

// WriteRequest {{{

pub struct WriteRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	handle: u64,
	offset: u64,
	write_flags: u32,
	lock_owner: u64,
	flags: u32,
	value: &'a [u8],
}

impl WriteRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn offset(&self) -> u64 {
		self.offset
	}

	pub fn value(&self) -> &[u8] {
		self.value
	}

	pub fn page_cache(&self) -> bool {
		self.write_flags & fuse_kernel::FUSE_WRITE_CACHE != 0
	}

	pub fn lock_owner(&self) -> Option<u64> {
		if self.write_flags & fuse_kernel::FUSE_WRITE_LOCKOWNER == 0 {
			return None;
		}
		Some(self.lock_owner)
	}

	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl fmt::Debug for WriteRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteRequest")
			.field("header", self.header)
			.field("handle", &self.handle)
			.field("offset", &self.offset)
			.field("lock_owner", &self.lock_owner())
			.field("flags", &self.flags)
			.field("page_cache", &self.page_cache())
			.field("value", &self.value)
			.finish()
	}
}

#[repr(C)]
struct fuse_write_in_v7p1 {
	fh: u64,
	offset: u64,
	size: u32,
	write_flags: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for WriteRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_WRITE);

		if dec.version().minor() < 9 {
			let raw: &'a fuse_write_in_v7p1 = dec.next_sized()?;
			let value = dec.next_bytes(raw.size)?;
			return Ok(Self {
				header,
				handle: raw.fh,
				offset: raw.offset,
				write_flags: raw.write_flags,
				lock_owner: 0,
				flags: 0,
				value,
			});
		}

		let raw: &'a fuse_kernel::fuse_write_in = dec.next_sized()?;
		let value = dec.next_bytes(raw.size)?;
		Ok(Self {
			header,
			handle: raw.fh,
			offset: raw.offset,
			write_flags: raw.write_flags,
			lock_owner: raw.lock_owner,
			flags: raw.flags,
			value,
		})
	}
}

// }}}

// WriteResponse {{{

pub struct WriteResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_write_out,
}

impl<'a> WriteResponse<'a> {
	pub fn new() -> WriteResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_write_out {
				size: 0,
				padding: 0,
			},
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.raw.size = size;
	}
}

impl fmt::Debug for WriteResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteResponse")
			.field("size", &self.raw.size)
			.finish()
	}
}

impl fuse_io::EncodeResponse for WriteResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_sized(&self.raw)
	}
}

// }}}
