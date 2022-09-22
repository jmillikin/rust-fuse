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

#[cfg(rust_fuse_test = "fsyncdir_test")]
mod fsyncdir_test;

// FsyncdirRequest {{{

const FSYNCDIR_DATASYNC: u32 = 1 << 0;

pub struct FsyncdirRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	handle: u64,
	flags: FsyncdirRequestFlags,
}

impl<'a> FsyncdirRequest<'a> {
	pub fn from_fuse_request(
		request: &FuseRequest<'a>,
	) -> Result<Self, RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_FSYNCDIR)?;

		let raw: &fuse_kernel::fuse_fsync_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(dec.header().nodeid)?,
			handle: raw.fh,
			flags: FsyncdirRequestFlags::from_bits(raw.fsync_flags),
		})
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn flags(&self) -> &FsyncdirRequestFlags {
		&self.flags
	}
}

bitflags_struct! {
	/// Optional flags set on [`FsyncdirRequest`].
	///
	/// [`FsyncdirRequest`]: struct.FsyncdirRequest.html
	pub struct FsyncdirRequestFlags(u32);

	FSYNCDIR_DATASYNC: datasync,
}

impl fmt::Debug for FsyncdirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("flags", &self.flags)
			.finish()
	}
}

// }}}

// FsyncdirResponse {{{

pub struct FsyncdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FsyncdirResponse<'a> {
	pub fn new() -> FsyncdirResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}

	response_send_funcs!();
}

impl fmt::Debug for FsyncdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirResponse").finish()
	}
}

impl FsyncdirResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &crate::server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_header_only()
	}
}

// }}}
