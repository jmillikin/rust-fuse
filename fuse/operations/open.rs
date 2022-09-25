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

//! Implements the `FUSE_OPEN` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;

// OpenRequest {{{

/// Request type for `FUSE_OPEN`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPEN` operation.
pub struct OpenRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	flags: u32,
}

impl<'a> OpenRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		decode_request(request.buf, false)
	}

	pub fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		decode_request(request.buf, true)
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	/// Platform-specific flags passed to [`open(2)`].
	///
	/// [`open(2)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/open.html
	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl fmt::Debug for OpenRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenRequest")
			.field("node_id", &self.node_id)
			.field("flags", &DebugHexU32(self.flags))
			.finish()
	}
}

fn decode_request<'a>(
	buf: decode::RequestBuf<'a>,
	is_cuse: bool,
) -> Result<OpenRequest<'a>, io::RequestError> {
	buf.expect_opcode(fuse_kernel::FUSE_OPEN)?;

	let node_id = if is_cuse {
		crate::ROOT_ID
	} else {
		decode::node_id(buf.header().nodeid)?
	};
	let mut dec = decode::RequestDecoder::new(buf);

	let raw: &'a fuse_kernel::fuse_open_in = dec.next_sized()?;
	Ok(OpenRequest {
		phantom: PhantomData,
		node_id,
		flags: raw.flags,
	})
}

// }}}

// OpenResponse {{{

/// Response type for `FUSE_OPEN`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPEN` operation.
pub struct OpenResponse<'a> {
	phantom: PhantomData<&'a ()>,
	handle: u64,
	flags: OpenResponseFlags,
}

impl<'a> OpenResponse<'a> {
	pub fn new() -> OpenResponse<'a> {
		Self {
			phantom: PhantomData,
			handle: 0,
			flags: OpenResponseFlags::new(),
		}
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.handle = handle;
	}

	pub fn flags(&self) -> &OpenResponseFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut OpenResponseFlags {
		&mut self.flags
	}
}

response_send_funcs!(OpenResponse<'_>);

impl fmt::Debug for OpenResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenResponse")
			.field("handle", &self.handle)
			.field("flags", &self.flags())
			.finish()
	}
}

impl OpenResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&fuse_kernel::fuse_open_out {
			fh: self.handle,
			open_flags: self.flags.to_bits(),
			padding: 0,
		})
	}
}

// }}}

// OpenResponseFlags {{{

bitflags_struct! {
	/// Optional flags set on [`OpenResponse`].
	pub struct OpenResponseFlags(u32);

	/// Use [page-based direct I/O][direct-io] on this file.
	///
	/// [direct-io]: https://lwn.net/Articles/348719/
	fuse_kernel::FOPEN_DIRECT_IO: direct_io,

	/// Allow the kernel to preserve cached file data from the last time this
	/// file was opened.
	fuse_kernel::FOPEN_KEEP_CACHE: keep_cache,

	/// Tell the kernel this file is not seekable.
	fuse_kernel::FOPEN_NONSEEKABLE: nonseekable,
}

// }}}
