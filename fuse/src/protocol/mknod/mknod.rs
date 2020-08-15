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
mod mknod_test;

// MknodRequest {{{

pub struct MknodRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	name: &'a CStr,
	mode: u32,
	rdev: u32,
	umask: u32,
}

impl MknodRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn name(&self) -> &CStr {
		self.name
	}

	pub fn mode(&self) -> u32 {
		self.mode
	}

	pub fn rdev(&self) -> u32 {
		self.rdev
	}

	pub fn umask(&self) -> u32 {
		self.umask
	}
}

#[repr(C)]
pub(crate) struct fuse_mknod_in_v7p1 {
	pub mode: u32,
	pub rdev: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for MknodRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_MKNOD);

		if dec.version().minor() < 12 {
			let raw: &fuse_mknod_in_v7p1 = dec.next_sized()?;
			let name = dec.next_cstr()?;
			return Ok(Self {
				header,
				name,
				mode: raw.mode,
				rdev: raw.rdev,
				umask: 0,
			});
		}

		let raw: &fuse_kernel::fuse_mknod_in = dec.next_sized()?;
		let name = dec.next_cstr()?;
		Ok(Self {
			header,
			name,
			mode: raw.mode,
			rdev: raw.rdev,
			umask: raw.umask,
		})
	}
}

// }}}

// MknodResponse {{{

pub struct MknodResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> MknodResponse<'a> {
	pub fn new() -> MknodResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: Default::default(),
		}
	}

	pub fn node(&self) -> &Node {
		Node::new_ref(&self.raw)
	}

	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.raw)
	}
}

impl fmt::Debug for MknodResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MknodResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl fuse_io::EncodeResponse for MknodResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		self.node().encode_entry(enc)
	}
}

// }}}
