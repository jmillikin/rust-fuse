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

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugBytesAsString;
use crate::protocol::common::DebugHexU32;

// WriteRequest {{{

/// Request type for `FUSE_WRITE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_WRITE` operation.
pub struct WriteRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	offset: u64,
	handle: u64,
	value: &'a [u8],
	flags: WriteRequestFlags,
	lock_owner: Option<u64>,
	open_flags: u32,
}

#[repr(C)]
struct fuse_write_in_v7p1 {
	fh: u64,
	offset: u64,
	size: u32,
	write_flags: u32,
}

impl<'a> WriteRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		decode_request(request.buf, request.version_minor, false)
	}

	pub fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		decode_request(request.buf, request.version_minor, true)
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn offset(&self) -> u64 {
		self.offset
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: crate::operations::open::OpenResponse::set_handle
	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn value(&self) -> &[u8] {
		self.value
	}

	pub fn flags(&self) -> WriteRequestFlags {
		self.flags
	}

	pub fn lock_owner(&self) -> Option<u64> {
		self.lock_owner
	}

	pub fn open_flags(&self) -> crate::OpenFlags {
		self.open_flags
	}
}

impl fmt::Debug for WriteRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteRequest")
			.field("node_id", &self.node_id)
			.field("offset", &self.offset)
			.field("handle", &self.handle)
			.field("value", &DebugBytesAsString(self.value))
			.field("flags", &self.flags)
			.field("lock_owner", &format_args!("{:?}", &self.lock_owner))
			.field("open_flags", &DebugHexU32(self.open_flags))
			.finish()
	}
}

fn decode_request<'a>(
	buf: decode::RequestBuf<'a>,
	version_minor: u32,
	is_cuse: bool,
) -> Result<WriteRequest<'a>, io::RequestError> {
	buf.expect_opcode(fuse_kernel::FUSE_WRITE)?;

	let node_id = if is_cuse {
		crate::ROOT_ID
	} else {
		decode::node_id(buf.header().nodeid)?
	};

	let mut dec = decode::RequestDecoder::new(buf);
	if version_minor < 9 {
		let raw: &'a fuse_write_in_v7p1 = dec.next_sized()?;
		let value = dec.next_bytes(raw.size)?;
		return Ok(WriteRequest {
			phantom: PhantomData,
			node_id,
			offset: raw.offset,
			handle: raw.fh,
			value,
			flags: WriteRequestFlags {
				bits: raw.write_flags,
			},
			lock_owner: None,
			open_flags: 0,
		});
	}

	let raw: &'a fuse_kernel::fuse_write_in = dec.next_sized()?;
	let value = dec.next_bytes(raw.size)?;

	let mut lock_owner = None;
	if raw.write_flags & fuse_kernel::FUSE_WRITE_LOCKOWNER != 0 {
		lock_owner = Some(raw.lock_owner)
	}

	Ok(WriteRequest {
		phantom: PhantomData,
		node_id,
		offset: raw.offset,
		handle: raw.fh,
		value,
		flags: WriteRequestFlags {
			bits: raw.write_flags,
		},
		lock_owner,
		open_flags: raw.flags,
	})
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
