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

use core::fmt;
use core::marker::PhantomData;

use crate::kernel;
use crate::server::decode;

// StatfsRequest {{{

/// Request type for `FUSE_STATFS`.
pub struct StatfsRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: crate::NodeId,
}

impl StatfsRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}
}

try_from_fuse_request!(StatfsRequest<'a>, |request| {
	let dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_STATFS)?;

	Ok(Self {
		phantom: PhantomData,
		node_id: decode::node_id(dec.header().nodeid)?,
	})
});

impl fmt::Debug for StatfsRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsRequest")
			.field("node_id", &self.node_id)
			.finish()
	}
}

// }}}

// StatfsAttributes {{{

#[derive(Clone, Copy)]
pub struct StatfsAttributes {
	raw: kernel::fuse_kstatfs,
}

impl StatfsAttributes {
	#[must_use]
	pub fn new() -> StatfsAttributes {
		Self {
			raw: kernel::fuse_kstatfs::new(),
		}
	}

	#[inline]
	#[must_use]
	pub fn raw(&self) -> &kernel::fuse_kstatfs {
		&self.raw
	}

	#[must_use]
	pub fn block_count(&self) -> u64 {
		self.raw.blocks
	}

	pub fn set_block_count(&mut self, block_count: u64) {
		self.raw.blocks = block_count;
	}

	#[must_use]
	pub fn block_size(&self) -> u32 {
		self.raw.bsize
	}

	pub fn set_block_size(&mut self, block_size: u32) {
		self.raw.bsize = block_size;
	}

	#[must_use]
	pub fn blocks_available(&self) -> u64 {
		self.raw.bavail
	}

	pub fn set_blocks_available(&mut self, blocks_available: u64) {
		self.raw.bavail = blocks_available;
	}

	#[must_use]
	pub fn blocks_free(&self) -> u64 {
		self.raw.bfree
	}

	pub fn set_blocks_free(&mut self, blocks_free: u64) {
		self.raw.bfree = blocks_free;
	}

	#[must_use]
	pub fn fragment_size(&self) -> u32 {
		self.raw.frsize
	}

	pub fn set_fragment_size(&mut self, fragment_size: u32) {
		self.raw.frsize = fragment_size;
	}

	#[must_use]
	pub fn inode_count(&self) -> u64 {
		self.raw.files
	}

	pub fn set_inode_count(&mut self, inode_count: u64) {
		self.raw.files = inode_count;
	}

	#[must_use]
	pub fn inodes_free(&self) -> u64 {
		self.raw.ffree
	}

	pub fn set_inodes_free(&mut self, inodes_free: u64) {
		self.raw.ffree = inodes_free;
	}

	#[must_use]
	pub fn max_filename_length(&self) -> u32 {
		self.raw.namelen
	}

	pub fn set_max_filename_length(&mut self, max_filename_length: u32) {
		self.raw.namelen = max_filename_length;
	}
}

impl fmt::Debug for StatfsAttributes {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsAttributes")
			.field("block_count", &self.block_count())
			.field("block_size", &self.block_size())
			.field("blocks_available", &self.blocks_available())
			.field("blocks_free", &self.blocks_free())
			.field("fragment_size", &self.fragment_size())
			.field("inode_count", &self.inode_count())
			.field("inodes_free", &self.inodes_free())
			.field("max_filename_length", &self.max_filename_length())
			.finish()
	}
}

// }}}
