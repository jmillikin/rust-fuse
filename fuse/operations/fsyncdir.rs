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

//! Implements the `FUSE_FSYNCDIR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// FsyncdirRequest {{{

/// Request type for `FUSE_FSYNCDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FSYNCDIR` operation.
pub struct FsyncdirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_fsync_in,
}

impl<'a> FsyncdirRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_FSYNCDIR)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}

	#[must_use]
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub fn flags(&self) -> FsyncdirRequestFlags {
		FsyncdirRequestFlags {
			bits: self.body.fsync_flags,
		}
	}
}

impl fmt::Debug for FsyncdirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// FsyncdirResponse {{{

/// Response type for `FUSE_FSYNCDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FSYNCDIR` operation.
pub struct FsyncdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FsyncdirResponse<'a> {
	#[must_use]
	pub fn new() -> FsyncdirResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(FsyncdirResponse<'_>);

impl fmt::Debug for FsyncdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirResponse").finish()
	}
}

impl FsyncdirResponse<'_> {
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

// FsyncdirRequestFlags {{{

/// Optional flags set on [`FsyncdirRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncdirRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncdirRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(FsyncdirRequestFlag, FsyncdirRequestFlags, u32, {
		FDATASYNC = fuse_kernel::FUSE_FSYNC_FDATASYNC;
	});
}

// }}}
