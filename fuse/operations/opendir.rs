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

//! Implements the `FUSE_OPENDIR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;

// OpendirRequest {{{

/// Request type for `FUSE_OPENDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPENDIR` operation.
pub struct OpendirRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	flags: u32,
}

impl<'a> OpendirRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_OPENDIR)?;
		let raw: &'a fuse_kernel::fuse_open_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: decode::node_id(dec.header().nodeid)?,
			flags: raw.flags,
		})
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

impl fmt::Debug for OpendirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpendirRequest")
			.field("node_id", &self.node_id)
			.field("flags", &DebugHexU32(self.flags))
			.finish()
	}
}

// }}}

// OpendirResponse {{{

/// Response type for `FUSE_OPENDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPENDIR` operation.
pub struct OpendirResponse<'a> {
	phantom: PhantomData<&'a ()>,
	handle: u64,
	flags: OpendirResponseFlags,
}

impl<'a> OpendirResponse<'a> {
	pub fn new() -> OpendirResponse<'a> {
		Self {
			phantom: PhantomData,
			handle: 0,
			flags: OpendirResponseFlags::new(),
		}
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.handle = handle;
	}

	pub fn flags(&self) -> &OpendirResponseFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut OpendirResponseFlags {
		&mut self.flags
	}
}

response_send_funcs!(OpendirResponse<'_>);

impl fmt::Debug for OpendirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpendirResponse")
			.field("handle", &self.handle)
			.field("flags", &self.flags())
			.finish()
	}
}
impl OpendirResponse<'_> {
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

// OpendirResponseFlags {{{

bitflags_struct! {
	/// Optional flags set on [`OpendirResponse`].
	pub struct OpendirResponseFlags(u32);

	/// Allow the kernel to preserve cached directory entries from the last
	/// time this directory was opened.
	fuse_kernel::FOPEN_KEEP_CACHE: keep_cache,

	/// Tell the kernel this directory is not seekable.
	fuse_kernel::FOPEN_NONSEEKABLE: nonseekable,

	// TODO: CACHE_DIR
}

// }}}
