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

//! Implements the `FUSE_RELEASE` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugHexU32;

// ReleaseRequest {{{

/// Request type for `FUSE_RELEASE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASE` operation.
pub struct ReleaseRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_release_in<'a>>,
}

impl ReleaseRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		node::Id::new(self.header.nodeid).unwrap_or(node::Id::ROOT)
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: crate::operations::open::OpenResponse::set_handle
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.as_v7p1().fh
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<u64> {
		let body = self.body.as_v7p8()?;
		if body.release_flags & fuse_kernel::FUSE_RELEASE_FLOCK_UNLOCK == 0 {
			return None;
		}
		Some(body.lock_owner)
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.as_v7p1().flags
	}
}

request_try_from! { ReleaseRequest : cuse fuse }

impl decode::Sealed for ReleaseRequest<'_> {}

impl<'a> decode::CuseRequest<'a> for ReleaseRequest<'a> {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, request.version_minor, true)
	}
}

impl<'a> decode::FuseRequest<'a> for ReleaseRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, request.version_minor, false)
	}
}

impl<'a> ReleaseRequest<'a> {
	fn decode_request(
		buf: decode::RequestBuf<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = decode::RequestDecoder::new(buf);
		dec.expect_opcode(fuse_kernel::FUSE_RELEASE)?;

		let header = dec.header();
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let body = if version_minor >= 8 {
			let body_v7p8 = dec.next_sized()?;
			compat::Versioned::new_release_v7p8(version_minor, body_v7p8)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_release_v7p1(version_minor, body_v7p1)
		};

		Ok(Self { header, body })
	}
}

impl fmt::Debug for ReleaseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("open_flags", &DebugHexU32(self.open_flags()))
			.finish()
	}
}

// }}}

// ReleaseResponse {{{

/// Response type for `FUSE_RELEASE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASE` operation.
pub struct ReleaseResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> ReleaseResponse<'a> {
	#[must_use]
	pub fn new() -> ReleaseResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(ReleaseResponse<'_>);

impl fmt::Debug for ReleaseResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseResponse").finish()
	}
}

impl ReleaseResponse<'_> {
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
