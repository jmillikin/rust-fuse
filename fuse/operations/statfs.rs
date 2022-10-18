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
	raw: fuse_kernel::fuse_statfs_out,
}

impl StatfsResponse {
	#[must_use]
	pub fn new() -> StatfsResponse {
		Self {
			raw: fuse_kernel::fuse_statfs_out::zeroed(),
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

impl fmt::Debug for StatfsResponse {
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
			return encode::sized(header, &self.raw);
		}

		let raw_ptr = &self.raw as *const fuse_kernel::fuse_statfs_out;
		encode::sized(header, unsafe {
			&*(raw_ptr.cast::<fuse_statfs_out_v7p1>())
		})
	}
}

// }}}
