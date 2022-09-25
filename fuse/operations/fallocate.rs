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
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// FallocateRequest {{{

const FALLOC_FL_KEEP_SIZE: u32 = 1 << 0;
const FALLOC_FL_PUNCH_HOLE: u32 = 1 << 1;
const FALLOC_FL_COLLAPSE_RANGE: u32 = 1 << 3;
const FALLOC_FL_ZERO_RANGE: u32 = 1 << 4;
const FALLOC_FL_INSERT_RANGE: u32 = 1 << 5;
const FALLOC_FL_UNSHARE_RANGE: u32 = 1 << 6;

/// Request type for `FUSE_FALLOCATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FALLOCATE` operation.
pub struct FallocateRequest<'a> {
	raw: &'a fuse_kernel::fuse_fallocate_in,
	node_id: NodeId,
	mode: FallocateMode,
}

impl<'a> FallocateRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_FALLOCATE)?;

		let raw = dec.next_sized()?;
		Ok(Self {
			raw,
			node_id: decode::node_id(dec.header().nodeid)?,
			mode: FallocateMode::from_bits(raw.mode),
		})
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	pub fn length(&self) -> u64 {
		self.raw.length
	}

	pub fn mode(&self) -> FallocateMode {
		self.mode
	}
}

bitflags_struct! {
	/// Mode bits set in an [`FallocateRequest`].
	pub struct FallocateMode(u32);

	FALLOC_FL_KEEP_SIZE: keep_size,
	FALLOC_FL_PUNCH_HOLE: punch_hole,
	FALLOC_FL_COLLAPSE_RANGE: collapse_range,
	FALLOC_FL_ZERO_RANGE: zero_range,
	FALLOC_FL_INSERT_RANGE: insert_range,
	FALLOC_FL_UNSHARE_RANGE: unshare_range,
}

impl fmt::Debug for FallocateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("offset", &self.raw.offset)
			.field("length", &self.raw.length)
			.field("mode", &self.mode)
			.finish()
	}
}

// }}}

// FallocateResponse {{{

/// Response type for `FUSE_FALLOCATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FALLOCATE` operation.
pub struct FallocateResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FallocateResponse<'a> {
	pub fn new() -> FallocateResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(FallocateResponse<'_>);

impl fmt::Debug for FallocateResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FallocateResponse").finish()
	}
}

impl FallocateResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_header_only()
	}
}

// }}}
