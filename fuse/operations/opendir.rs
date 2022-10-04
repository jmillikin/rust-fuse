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
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugHexU32;

// OpendirRequest {{{

/// Request type for `FUSE_OPENDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPENDIR` operation.
pub struct OpendirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_open_in,
}

impl OpendirRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn flags(&self) -> OpendirRequestFlags {
		OpendirRequestFlags {
			bits: self.body.open_flags,
		}
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.flags
	}
}

request_try_from! { OpendirRequest : fuse }

impl decode::Sealed for OpendirRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for OpendirRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_OPENDIR)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for OpendirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpendirRequest")
			.field("node_id", &self.node_id())
			.field("flags", &self.flags())
			.field("open_flags", &DebugHexU32(self.open_flags()))
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
	raw: fuse_kernel::fuse_open_out,
}

impl<'a> OpendirResponse<'a> {
	#[must_use]
	pub fn new() -> OpendirResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_open_out::zeroed(),
		}
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.raw.fh = handle;
	}

	#[must_use]
	pub fn flags(&self) -> OpendirResponseFlags {
		OpendirResponseFlags {
			bits: self.raw.open_flags,
		}
	}

	#[must_use]
	pub fn mut_flags(&mut self) -> &mut OpendirResponseFlags {
		OpendirResponseFlags::reborrow_mut(&mut self.raw.open_flags)
	}

	pub fn set_flags(&mut self, flags: OpendirResponseFlags) {
		self.raw.open_flags = flags.bits
	}
}

response_send_funcs!(OpendirResponse<'_>);

impl fmt::Debug for OpendirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpendirResponse")
			.field("handle", &self.handle())
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
		enc.encode_sized(&self.raw)
	}
}

// }}}

// OpendirRequestFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(OpendirRequestFlag, OpendirRequestFlags, u32, {
		KILL_SUIDGID = fuse_kernel::FUSE_OPEN_KILL_SUIDGID;
	});
}

// }}}

// OpendirResponseFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirResponseFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirResponseFlag {
	mask: u32,
}

mod response_flags {
	use crate::internal::fuse_kernel;
	bitflags!(OpendirResponseFlag, OpendirResponseFlags, u32, {
		DIRECT_IO = fuse_kernel::FOPEN_DIRECT_IO;
		KEEP_CACHE = fuse_kernel::FOPEN_KEEP_CACHE;
		NONSEEKABLE = fuse_kernel::FOPEN_NONSEEKABLE;
		CACHE_DIR = fuse_kernel::FOPEN_CACHE_DIR;
		STREAM = fuse_kernel::FOPEN_STREAM;
		NOFLUSH = fuse_kernel::FOPEN_NOFLUSH;
	});
}

// }}}
