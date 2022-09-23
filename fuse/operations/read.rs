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

#[cfg(rust_fuse_test = "read_test")]
mod read_test;

// ReadRequest {{{

/// Request type for [`FuseHandlers::read`].
///
/// [`FuseHandlers::read`]: ../../trait.FuseHandlers.html#method.read
pub struct ReadRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	size: u32,
	offset: u64,
	handle: u64,
	lock_owner: Option<u64>,
	open_flags: u32,
}

#[repr(C)]
pub(crate) struct fuse_read_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) offset: u64,
	pub(crate) size: u32,
	pub(crate) padding: u32,
}

impl<'a> ReadRequest<'a> {
	pub fn from_fuse_request(
		request: &FuseRequest<'a>,
	) -> Result<Self, RequestError> {
		decode_request(request.buf, request.version_minor, false)
	}

	pub fn from_cuse_request(
		request: &CuseRequest<'a>,
	) -> Result<Self, RequestError> {
		decode_request(request.buf, request.version_minor, true)
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	pub fn offset(&self) -> u64 {
		self.offset
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: struct.OpenResponse.html#method.set_handle
	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn lock_owner(&self) -> Option<u64> {
		self.lock_owner
	}

	/// Platform-specific flags passed to [`FuseHandlers::open`]. See
	/// [`OpenRequest::flags`] for details.
	///
	/// [`FuseHandlers::open`]: ../../trait.FuseHandlers.html#method.open
	/// [`OpenRequest::flags`]: struct.OpenRequest.html#method.flags
	pub fn open_flags(&self) -> u32 {
		self.open_flags
	}
}

impl fmt::Debug for ReadRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadRequest")
			.field("node_id", &self.node_id)
			.field("size", &self.size)
			.field("offset", &self.offset)
			.field("handle", &self.handle)
			.field("lock_owner", &format_args!("{:?}", &self.lock_owner))
			.field("open_flags", &DebugHexU32(self.open_flags))
			.finish()
	}
}

fn decode_request<'a>(
	buf: decode::RequestBuf<'a>,
	version_minor: u32,
	is_cuse: bool,
) -> Result<ReadRequest<'a>, io::RequestError> {
	buf.expect_opcode(fuse_kernel::FUSE_READ)?;

	let node_id = if is_cuse {
		crate::ROOT_ID
	} else {
		try_node_id(buf.header().nodeid)?
	};
	let mut dec = decode::RequestDecoder::new(buf);

	// FUSE v7.9 added new fields to `fuse_read_in`.
	if version_minor < 9 {
		let raw: &'a fuse_read_in_v7p1 = dec.next_sized()?;
		return Ok(ReadRequest {
			phantom: PhantomData,
			node_id,
			size: raw.size,
			offset: raw.offset,
			handle: raw.fh,
			lock_owner: None,
			open_flags: 0,
		});
	}

	let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;

	let mut lock_owner = None;
	if raw.read_flags & fuse_kernel::FUSE_READ_LOCKOWNER != 0 {
		lock_owner = Some(raw.lock_owner);
	}

	Ok(ReadRequest {
		phantom: PhantomData,
		node_id,
		size: raw.size,
		offset: raw.offset,
		handle: raw.fh,
		lock_owner,
		open_flags: raw.flags,
	})
}

// }}}

// ReadResponse {{{

/// Response type for [`FuseHandlers::read`].
///
/// [`FuseHandlers::read`]: ../../trait.FuseHandlers.html#method.read
pub struct ReadResponse<'a> {
	bytes: &'a [u8],
}

impl<'a> ReadResponse<'a> {
	pub fn from_bytes(bytes: &'a [u8]) -> ReadResponse<'a> {
		Self { bytes }
	}

	// TODO; from &[std::io::IoSlice]

	// TODO: from file descriptor (for splicing)

	response_send_funcs!();
}

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
		ctx: &crate::server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_bytes(self.bytes)
	}
}

// }}}
