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

//! Implements the `FUSE_FSYNC` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// FsyncRequest {{{

/// Request type for `FUSE_FSYNC`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FSYNC` operation.
pub struct FsyncRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_fsync_in,
}

impl FsyncRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> NodeId {
		match NodeId::new(self.header.nodeid) {
			Some(id) => id,
			None => crate::ROOT_ID,
		}
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub fn flags(&self) -> FsyncRequestFlags {
		FsyncRequestFlags {
			bits: self.body.fsync_flags,
		}
	}
}

request_try_from! { FsyncRequest : cuse fuse }

impl decode::Sealed for FsyncRequest<'_> {}

impl<'a> decode::CuseRequest<'a> for FsyncRequest<'a> {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, true)
	}
}

impl<'a> decode::FuseRequest<'a> for FsyncRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, false)
	}
}

impl<'a> FsyncRequest<'a> {
	fn decode_request(
		buf: decode::RequestBuf<'a>,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = decode::RequestDecoder::new(buf);
		dec.expect_opcode(fuse_kernel::FUSE_FSYNC)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}
		Ok(Self { header, body })
	}
}

impl fmt::Debug for FsyncRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// FsyncResponse {{{

/// Response type for `FUSE_FSYNC`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FSYNC` operation.
pub struct FsyncResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FsyncResponse<'a> {
	#[must_use]
	pub fn new() -> FsyncResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(FsyncResponse<'_>);

impl fmt::Debug for FsyncResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncResponse").finish()
	}
}

impl FsyncResponse<'_> {
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

// FsyncRequestFlags {{{

/// Optional flags set on [`FsyncRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(FsyncRequestFlag, FsyncRequestFlags, u32, {
		FDATASYNC = fuse_kernel::FUSE_FSYNC_FDATASYNC;
	});
}

// }}}
