// Copyright 2022 John Millikin and the rust-fuse contributors.
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

//! Implements the `FUSE_COPY_FILE_RANGE` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// CopyFileRangeRequest {{{

/// Request type for `FUSE_COPY_FILE_RANGE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_COPY_FILE_RANGE` operation.
pub struct CopyFileRangeRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_copy_file_range_in,
}

impl<'a> CopyFileRangeRequest<'a> {
	pub fn input_node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	pub fn input_handle(&self) -> u64 {
		self.body.fh_in
	}

	pub fn input_offset(&self) -> u64 {
		self.body.off_in
	}

	pub fn output_node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.body.nodeid_out) }
	}

	pub fn output_handle(&self) -> u64 {
		self.body.fh_out
	}

	pub fn output_offset(&self) -> u64 {
		self.body.off_out
	}

	pub fn len(&self) -> u64 {
		self.body.len
	}

	pub fn flags(&self) -> CopyFileRangeRequestFlags {
		CopyFileRangeRequestFlags {
			bits: self.body.flags,
		}
	}

	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_COPY_FILE_RANGE)?;

		use fuse_kernel::fuse_copy_file_range_in;

		let header = dec.header();
		let body: &'a fuse_copy_file_range_in = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		decode::node_id(body.nodeid_out)?;

		Ok(Self { header, body })
	}
}

impl fmt::Debug for CopyFileRangeRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CopyFileRangeRequest")
			.field("input_node_id", &self.input_node_id())
			.field("input_handle", &self.input_handle())
			.field("input_offset", &self.input_offset())
			.field("output_node_id", &self.output_node_id())
			.field("output_handle", &self.output_handle())
			.field("output_offset", &self.output_offset())
			.field("len", &self.len())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// CopyFileRangeResponse {{{

/// Response type for `FUSE_COPY_FILE_RANGE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_COPY_FILE_RANGE` operation.
pub struct CopyFileRangeResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_write_out,
}

impl<'a> CopyFileRangeResponse<'a> {
	pub fn new() -> CopyFileRangeResponse<'a> {
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

response_send_funcs!(CopyFileRangeResponse<'_>);

impl fmt::Debug for CopyFileRangeResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CopyFileRangeResponse")
			.field("size", &self.raw.size)
			.finish()
	}
}

impl CopyFileRangeResponse<'_> {
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

// CopyFileRangeRequestFlags {{{

/// Optional flags set on [`CopyFileRangeRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CopyFileRangeRequestFlags {
	bits: u64,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CopyFileRangeRequestFlag {
	mask: u64,
}

mod request_flags {
	bitflags!(CopyFileRangeRequestFlag, CopyFileRangeRequestFlags, u64, {
	});
}

// }}}
