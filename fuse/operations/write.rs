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

//! Implements the `FUSE_WRITE` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::compat;
use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::lock;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// WriteRequest {{{

/// Request type for `FUSE_WRITE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_WRITE` operation.
pub struct WriteRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_write_in<'a>>,
	value: &'a [u8],
}

impl WriteRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		node::Id::new(self.header.nodeid).unwrap_or(node::Id::ROOT)
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
	pub fn value(&self) -> &[u8] {
		self.value
	}

	#[must_use]
	pub fn flags(&self) -> WriteRequestFlags {
		WriteRequestFlags {
			bits: self.body.as_v7p1().write_flags,
		}
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<lock::Owner> {
		let body = self.body.as_v7p9()?;
		if body.write_flags & fuse_kernel::FUSE_WRITE_LOCKOWNER == 0 {
			return None;
		}
		Some(lock::Owner::new(body.lock_owner))
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		if let Some(body) = self.body.as_v7p9() {
			return body.flags;
		}
		0
	}
}

request_try_from! { WriteRequest : cuse fuse }

impl decode::Sealed for WriteRequest<'_> {}

impl<'a> decode::CuseRequest<'a> for WriteRequest<'a> {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, request.version_minor, true)
	}
}

impl<'a> decode::FuseRequest<'a> for WriteRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request.buf, request.version_minor, false)
	}
}

impl<'a> WriteRequest<'a> {
	fn decode_request(
		buf: decode::RequestBuf<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = decode::RequestDecoder::new(buf);
		dec.expect_opcode(fuse_kernel::FUSE_WRITE)?;

		let header = dec.header();
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let body = if version_minor >= 9 {
			let body_v7p9 = dec.next_sized()?;
			compat::Versioned::new_write_v7p9(version_minor, body_v7p9)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_write_v7p1(version_minor, body_v7p1)
		};

		let value = dec.next_bytes(body.as_v7p1().size)?;

		Ok(Self { header, body, value })
	}
}

impl fmt::Debug for WriteRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteRequest")
			.field("node_id", &self.node_id())
			.field("offset", &self.offset())
			.field("handle", &self.handle())
			.field("value", &debug::bytes(self.value))
			.field("flags", &self.flags())
			.field("lock_owner", &format_args!("{:?}", &self.lock_owner()))
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}

// WriteResponse {{{

/// Response type for `FUSE_WRITE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_WRITE` operation.
pub struct WriteResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_write_out,
}

impl<'a> WriteResponse<'a> {
	#[must_use]
	pub fn new() -> WriteResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_write_out {
				size: 0,
				padding: 0,
			},
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.raw.size = size;
	}
}

response_send_funcs!(WriteResponse<'_>);

impl fmt::Debug for WriteResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteResponse")
			.field("size", &self.raw.size)
			.finish()
	}
}

impl WriteResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&self.raw)
	}
}

// }}}

// WriteRequestFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WriteRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WriteRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(WriteRequestFlag, WriteRequestFlags, u32, {
		WRITE_CACHE = fuse_kernel::FUSE_WRITE_CACHE;
		WRITE_LOCKOWNER = fuse_kernel::FUSE_WRITE_LOCKOWNER;
		WRITE_KILL_SUIDGID = fuse_kernel::FUSE_WRITE_KILL_SUIDGID;
	});
}

// }}}
