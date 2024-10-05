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

//! Implements the `FUSE_BMAP` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// BmapRequest {{{

/// Request type for `FUSE_BMAP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_BMAP` operation.
pub struct BmapRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_bmap_in,
}

impl BmapRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn block(&self) -> u64 {
		self.body.block
	}

	#[must_use]
	pub fn block_size(&self) -> u32 {
		self.body.blocksize
	}
}

impl server::sealed::Sealed for BmapRequest<'_> {}

impl<'a> server::FuseRequest<'a> for BmapRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_BMAP)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for BmapRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("BmapRequest")
			.field("node_id", &self.node_id())
			.field("block", &self.block())
			.field("block_size", &self.block_size())
			.finish()
	}
}

// }}}

// BmapResponse {{{

/// Response type for `FUSE_BMAP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_BMAP` operation.
pub struct BmapResponse {
	raw: fuse_kernel::fuse_bmap_out,
}

impl BmapResponse {
	#[must_use]
	pub fn new() -> BmapResponse {
		Self {
			raw: fuse_kernel::fuse_bmap_out::zeroed(),
		}
	}

	#[must_use]
	pub fn block(self) -> u64 {
		self.raw.block
	}

	pub fn set_block(&mut self, block: u64) {
		self.raw.block = block;
	}
}

impl fmt::Debug for BmapResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("BmapResponse")
			.field("block", &self.raw.block)
			.finish()
	}
}

impl server::sealed::Sealed for BmapResponse {}

impl server::FuseResponse for BmapResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
	}
}

// }}}
