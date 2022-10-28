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

//! Implements the `FUSE_STATFS` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// StatfsRequest {{{

/// Request type for `FUSE_STATFS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_STATFS` operation.
pub struct StatfsRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: node::Id,
}

impl StatfsRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		self.node_id
	}
}

impl server::sealed::Sealed for StatfsRequest<'_> {}

impl<'a> server::FuseRequest<'a> for StatfsRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_STATFS)?;

		Ok(Self {
			phantom: PhantomData,
			node_id: decode::node_id(dec.header().nodeid)?,
		})
	}
}

impl fmt::Debug for StatfsRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsRequest")
			.field("node_id", &self.node_id)
			.finish()
	}
}

// }}}

// StatfsResponse {{{

/// Response type for `FUSE_STATFS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_STATFS` operation.
pub struct StatfsResponse {
	attr: StatfsAttributes,
}

impl StatfsResponse {
	#[must_use]
	pub fn new(attr: StatfsAttributes) -> StatfsResponse {
		Self { attr }
	}

	#[must_use]
	pub fn attributes(&self) -> &StatfsAttributes {
		&self.attr
	}

	#[must_use]
	pub fn mut_attributes(&mut self) -> &mut StatfsAttributes {
		&mut self.attr
	}
}

impl fmt::Debug for StatfsResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsResponse")
			.field("attributes", &self.attributes())
			.finish()
	}
}

impl server::sealed::Sealed for StatfsResponse {}

#[repr(C)]
struct fuse_statfs_out_v7p1 {
	blocks: u64,
	bfree: u64,
	bavail: u64,
	files: u64,
	ffree: u64,
	bsize: u32,
	namelen: u32,
}

impl server::FuseResponse for StatfsResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		if options.version_minor() >= 4 {
			return encode::sized(header, &self.attr.raw);
		}

		let raw_ptr = &self.attr.raw as *const fuse_kernel::fuse_statfs_out;
		encode::sized(header, unsafe {
			&*(raw_ptr.cast::<fuse_statfs_out_v7p1>())
		})
	}
}

// }}}

// StatfsAttributes {{{

#[derive(Clone, Copy)]
pub struct StatfsAttributes {
	raw: fuse_kernel::fuse_statfs_out,
}

impl StatfsAttributes {
	#[must_use]
	pub fn new() -> StatfsAttributes {
		Self {
			raw: fuse_kernel::fuse_statfs_out::zeroed(),
		}
	}

	#[must_use]
	pub fn block_count(&self) -> u64 {
		self.raw.st.blocks
	}

	pub fn set_block_count(&mut self, block_count: u64) {
		self.raw.st.blocks = block_count;
	}

	#[must_use]
	pub fn block_size(&self) -> u32 {
		self.raw.st.bsize
	}

	pub fn set_block_size(&mut self, block_size: u32) {
		self.raw.st.bsize = block_size;
	}

	#[must_use]
	pub fn blocks_available(&self) -> u64 {
		self.raw.st.bavail
	}

	pub fn set_blocks_available(&mut self, blocks_available: u64) {
		self.raw.st.bavail = blocks_available;
	}

	#[must_use]
	pub fn blocks_free(&self) -> u64 {
		self.raw.st.bfree
	}

	pub fn set_blocks_free(&mut self, blocks_free: u64) {
		self.raw.st.bfree = blocks_free;
	}

	#[must_use]
	pub fn fragment_size(&self) -> u32 {
		self.raw.st.frsize
	}

	pub fn set_fragment_size(&mut self, fragment_size: u32) {
		self.raw.st.frsize = fragment_size;
	}

	#[must_use]
	pub fn inode_count(&self) -> u64 {
		self.raw.st.files
	}

	pub fn set_inode_count(&mut self, inode_count: u64) {
		self.raw.st.files = inode_count;
	}

	#[must_use]
	pub fn inodes_free(&self) -> u64 {
		self.raw.st.ffree
	}

	pub fn set_inodes_free(&mut self, inodes_free: u64) {
		self.raw.st.ffree = inodes_free;
	}

	#[must_use]
	pub fn max_filename_length(&self) -> u32 {
		self.raw.st.namelen
	}

	pub fn set_max_filename_length(&mut self, max_filename_length: u32) {
		self.raw.st.namelen = max_filename_length;
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
