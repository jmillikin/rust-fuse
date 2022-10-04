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

//! Implements the `FUSE_READ` operation.

use core::fmt;

use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugBytesAsString;
use crate::protocol::common::DebugHexU32;

// ReadRequest {{{

/// Request type for `FUSE_READ`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READ` operation.
pub struct ReadRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_read_in<'a>>,
}

impl ReadRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		crate::NodeId::new(self.header.nodeid).unwrap_or(crate::ROOT_ID)
	}

	#[must_use]
	pub fn size(&self) -> u32 {
		self.body.as_v7p1().size
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.body.as_v7p1().offset
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
		let body = self.body.as_v7p9()?;
		if body.read_flags & fuse_kernel::FUSE_READ_LOCKOWNER == 0 {
			return None;
		}
		Some(body.lock_owner)
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		if let Some(body) = self.body.as_v7p9() {
			return body.flags;
		}
		0
	}
}

request_try_from! { ReadRequest : cuse fuse }

impl decode::Sealed for ReadRequest<'_> {}

impl<'a> decode::CuseRequest<'a> for ReadRequest<'a> {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, request.version_minor, true)
	}
}

impl<'a> decode::FuseRequest<'a> for ReadRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, request.version_minor, false)
	}
}

impl<'a> ReadRequest<'a> {
	fn decode_request(
		buf: decode::RequestBuf<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<ReadRequest<'a>, server::RequestError> {
		let mut dec = decode::RequestDecoder::new(buf);
		dec.expect_opcode(fuse_kernel::FUSE_READ)?;
		let header = dec.header();

		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let body = if version_minor >= 9 {
			let body_v7p9 = dec.next_sized()?;
			compat::Versioned::new_read_v7p9(version_minor, body_v7p9)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_read_v7p1(version_minor, body_v7p1)
		};

		Ok(ReadRequest { header, body })
	}
}

impl fmt::Debug for ReadRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadRequest")
			.field("node_id", &self.node_id())
			.field("size", &self.size())
			.field("offset", &self.offset())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", &self.lock_owner()))
			.field("open_flags", &DebugHexU32(self.open_flags()))
			.finish()
	}
}

// }}}

// ReadResponse {{{

/// Response type for `FUSE_READ`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READ` operation.
pub struct ReadResponse<'a> {
	bytes: &'a [u8],
}

impl<'a> ReadResponse<'a> {
	#[must_use]
	pub fn from_bytes(bytes: &'a [u8]) -> ReadResponse<'a> {
		Self { bytes }
	}

	// TODO; from &[std::io::IoSlice]

	// TODO: from file descriptor (for splicing)
}

response_send_funcs!(ReadResponse<'_>);

impl fmt::Debug for ReadResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let bytes = DebugBytesAsString(self.bytes);
		fmt.debug_struct("ReadResponse")
			.field("bytes", &bytes)
			.finish()
	}
}

impl ReadResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_bytes(self.bytes)
	}
}

// }}}
