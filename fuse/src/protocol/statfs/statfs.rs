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

#[cfg(rust_fuse_test = "statfs_test")]
mod statfs_test;

// StatfsRequest {{{

pub struct StatfsRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
}

impl StatfsRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
}

impl fmt::Debug for StatfsRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsRequest")
			.field("node_id", &self.node_id)
			.finish()
	}
}
impl<'a> decode::DecodeRequest<'a, decode::FUSE> for StatfsRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		let header = buf.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_STATFS);
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
		})
	}
}

// }}}

// StatfsResponse {{{

pub struct StatfsResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_statfs_out,
}

impl<'a> StatfsResponse<'a> {
	pub fn new() -> StatfsResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: Default::default(),
		}
	}

	pub fn set_block_count(&mut self, block_count: u64) {
		self.raw.st.blocks = block_count;
	}

	pub fn set_block_size(&mut self, block_size: u32) {
		self.raw.st.bsize = block_size;
	}

	pub fn set_blocks_available(&mut self, blocks_available: u64) {
		self.raw.st.bavail = blocks_available;
	}

	pub fn set_blocks_free(&mut self, blocks_free: u64) {
		self.raw.st.bfree = blocks_free;
	}

	pub fn set_fragment_size(&mut self, fragment_size: u32) {
		self.raw.st.frsize = fragment_size;
	}

	pub fn set_inode_count(&mut self, inode_count: u64) {
		self.raw.st.files = inode_count;
	}

	pub fn set_inodes_free(&mut self, inodes_free: u64) {
		self.raw.st.ffree = inodes_free;
	}

	pub fn set_max_filename_length(&mut self, max_filename_length: u32) {
		self.raw.st.namelen = max_filename_length;
	}
}

impl fmt::Debug for StatfsResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsResponse")
			.field("block_count", &self.raw.st.blocks)
			.field("block_size", &self.raw.st.bsize)
			.field("blocks_available", &self.raw.st.bavail)
			.field("blocks_free", &self.raw.st.bfree)
			.field("fragment_size", &self.raw.st.frsize)
			.field("inode_count", &self.raw.st.files)
			.field("inodes_free", &self.raw.st.ffree)
			.field("max_filename_length", &self.raw.st.namelen)
			.finish()
	}
}

impl fuse_io::EncodeResponse for StatfsResponse<'_> {
	fn encode_response<'a, S: io::OutputStream>(
		&'a self,
		enc: fuse_io::ResponseEncoder<S>,
	) -> Result<(), S::Error> {
		if enc.version().minor() < 4 {
			let buf: &[u8] = unsafe {
				slice::from_raw_parts(
					(&self.raw as *const fuse_kernel::fuse_statfs_out)
						as *const u8,
					fuse_kernel::FUSE_COMPAT_STATFS_SIZE,
				)
			};
			return enc.encode_bytes(buf);
		}
		enc.encode_sized(&self.raw)
	}
}

// }}}
