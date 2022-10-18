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

//! Implements the `FUSE_FALLOCATE` operation.

use core::fmt;

use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// FallocateRequest {{{

/// Request type for `FUSE_FALLOCATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FALLOCATE` operation.
pub struct FallocateRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_fallocate_in,
}

impl FallocateRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.body.offset
	}

	#[must_use]
	pub fn length(&self) -> u64 {
		self.body.length
	}

	#[must_use]
	pub fn fallocate_flags(&self) -> crate::FallocateFlags {
		self.body.mode
	}
}

impl server::sealed::Sealed for FallocateRequest<'_> {}

impl<'a> server::FuseRequest<'a> for FallocateRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_FALLOCATE)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for FallocateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("offset", &self.offset())
			.field("length", &self.length())
			.field("fallocate_flags", &debug::hex_u32(self.fallocate_flags()))
			.finish()
	}
}

// }}}

// FallocateResponse {{{

/// Response type for `FUSE_FALLOCATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FALLOCATE` operation.
pub struct FallocateResponse {
	_priv: (),
}

impl FallocateResponse {
	#[must_use]
	pub fn new() -> FallocateResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for FallocateResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateResponse").finish()
	}
}

impl server::sealed::Sealed for FallocateResponse {}

impl server::FuseResponse for FallocateResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
