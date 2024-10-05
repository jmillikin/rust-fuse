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
use core::mem;
use core::slice;

use crate::internal::compat;
use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::lock;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// WriteRequest {{{

/// Request type for `FUSE_WRITE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_WRITE` operation.
pub struct WriteRequest<'a> {
	msg: &'a write_msg,
	version_minor: u32,
}

#[repr(C)]
struct write_msg {
	header: fuse_kernel::fuse_in_header,
	body: fuse_write_in,
}

#[repr(C)]
union fuse_write_in {
	v7p1: compat::fuse_write_in_v7p1,
	v7p9: fuse_kernel::fuse_write_in,
}

const VALUE_OFFSET_V7P1: usize =
	  mem::size_of::<fuse_kernel::fuse_in_header>()
	+ mem::size_of::<compat::fuse_write_in_v7p1>();

const VALUE_OFFSET_V7P9: usize =
	  mem::size_of::<fuse_kernel::fuse_in_header>()
	+ mem::size_of::<fuse_kernel::fuse_write_in>();

impl<'a> WriteRequest<'a> {
	#[inline]
	fn header(&self) -> &'a fuse_kernel::fuse_in_header {
		&self.msg.header
	}

	#[inline]
	fn body_v7p1(&self) -> &'a compat::fuse_write_in_v7p1 {
		unsafe { &self.msg.body.v7p1 }
	}

	#[inline]
	fn body_v7p9(&self) -> Option<&'a fuse_kernel::fuse_write_in> {
		if self.version_minor >= 9 {
			return Some(unsafe { &self.msg.body.v7p9 });
		}
		None
	}
}

impl WriteRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		crate::NodeId::new(self.header().nodeid).unwrap_or(crate::NodeId::ROOT)
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.body_v7p1().offset
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: crate::operations::open::OpenResponse::set_handle
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body_v7p1().fh
	}

	#[must_use]
	pub fn value(&self) -> &[u8] {
		let value_len = self.body_v7p1().size;

		let offset = if self.version_minor >= 9 {
			VALUE_OFFSET_V7P9
		} else {
			VALUE_OFFSET_V7P1
		};

		unsafe {
			slice::from_raw_parts(
				((self.msg as *const write_msg).cast::<u8>()).add(offset),
				value_len as usize,
			)
		}
	}

	#[must_use]
	pub fn flags(&self) -> WriteRequestFlags {
		WriteRequestFlags {
			bits: self.body_v7p1().write_flags,
		}
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<lock::Owner> {
		let body = self.body_v7p9()?;
		if body.write_flags & fuse_kernel::FUSE_WRITE_LOCKOWNER == 0 {
			return None;
		}
		Some(lock::Owner::new(body.lock_owner))
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		if let Some(body) = self.body_v7p9() {
			return body.flags;
		}
		0
	}
}

impl server::sealed::Sealed for WriteRequest<'_> {}

impl<'a> server::CuseRequest<'a> for WriteRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		options: server::CuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request, options.version_minor(), true)
	}
}

impl<'a> server::FuseRequest<'a> for WriteRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request, options.version_minor(), false)
	}
}

impl<'a> WriteRequest<'a> {
	fn decode_request(
		request: server::Request<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_WRITE)?;

		let header = dec.header();
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let value_len = if version_minor >= 9 {
			let body: &fuse_kernel::fuse_write_in = dec.next_sized()?;
			body.size
		} else {
			let body: &compat::fuse_write_in_v7p1 = dec.next_sized()?;
			body.size
		};
		dec.next_bytes(value_len)?;

		let header_ptr = header as *const fuse_kernel::fuse_in_header;
		Ok(Self {
			msg: unsafe { &*(header_ptr.cast()) },
			version_minor,
		})
	}
}

impl fmt::Debug for WriteRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteRequest")
			.field("node_id", &self.node_id())
			.field("offset", &self.offset())
			.field("handle", &self.handle())
			.field("value", &debug::bytes(self.value()))
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
pub struct WriteResponse {
	raw: fuse_kernel::fuse_write_out,
}

impl WriteResponse {
	#[must_use]
	pub fn new() -> WriteResponse {
		Self {
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

impl fmt::Debug for WriteResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteResponse")
			.field("size", &self.raw.size)
			.finish()
	}
}

impl server::sealed::Sealed for WriteResponse {}

impl server::CuseResponse for WriteResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::CuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
	}
}

impl server::FuseResponse for WriteResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
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
