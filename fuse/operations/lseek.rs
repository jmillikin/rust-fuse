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

//! Implements the `FUSE_LSEEK` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// LseekRequest {{{

/// Request type for `FUSE_LSEEK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LSEEK` operation.
pub struct LseekRequest<'a> {
	raw: &'a fuse_kernel::fuse_lseek_in,
	node_id: node::Id,
}

impl LseekRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		self.node_id
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	#[must_use]
	pub fn whence(&self) -> LseekWhence {
		LseekWhence(self.raw.whence)
	}
}

impl server::sealed::Sealed for LseekRequest<'_> {}

impl<'a> server::FuseRequest<'a> for LseekRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LSEEK)?;
		let raw = dec.next_sized()?;
		Ok(Self {
			raw,
			node_id: decode::node_id(dec.header().nodeid)?,
		})
	}
}

impl fmt::Debug for LseekRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LseekRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("offset", &self.raw.offset)
			.field("whence", &LseekWhence(self.raw.whence))
			.finish()
	}
}

#[derive(Eq, PartialEq)]
pub struct LseekWhence(u32);

impl LseekWhence {
	pub const SEEK_DATA: LseekWhence = LseekWhence(3);
	pub const SEEK_HOLE: LseekWhence = LseekWhence(4);
}

impl fmt::Debug for LseekWhence {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self.0 {
			3 => fmt.write_str("SEEK_DATA"),
			4 => fmt.write_str("SEEK_HOLE"),
			_ => self.0.fmt(fmt),
		}
	}
}

// }}}

// LseekResponse {{{

/// Response type for `FUSE_LSEEK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LSEEK` operation.
pub struct LseekResponse {
	raw: fuse_kernel::fuse_lseek_out,
}

impl LseekResponse {
	#[must_use]
	pub fn new() -> LseekResponse {
		Self {
			raw: fuse_kernel::fuse_lseek_out { offset: 0 },
		}
	}

	pub fn set_offset(&mut self, offset: u64) {
		self.raw.offset = offset;
	}
}

impl fmt::Debug for LseekResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LseekResponse")
			.field("offset", &self.raw.offset)
			.finish()
	}
}

impl server::sealed::Sealed for LseekResponse {}

impl server::FuseResponse for LseekResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
	}
}

// }}}
