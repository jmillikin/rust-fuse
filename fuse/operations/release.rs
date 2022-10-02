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

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;

// ReleaseRequest {{{

/// Request type for `FUSE_RELEASE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASE` operation.
pub struct ReleaseRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	handle: u64,
	lock_owner: Option<u64>,
	open_flags: u32,
}

#[repr(C)]
pub(crate) struct fuse_release_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) flags: u32,
	pub(crate) padding: u32,
}

impl<'a> ReleaseRequest<'a> {
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

	#[must_use]
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: crate::operations::open::OpenResponse::set_handle
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.handle
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<u64> {
		self.lock_owner
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.open_flags
	}
}

impl fmt::Debug for ReleaseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.field("open_flags", &DebugHexU32(self.open_flags))
			.finish()
	}
}

fn decode_request<'a>(
	buf: decode::RequestBuf<'a>,
	version_minor: u32,
	is_cuse: bool,
) -> Result<ReleaseRequest<'a>, io::RequestError> {
	buf.expect_opcode(fuse_kernel::FUSE_RELEASE)?;

	let node_id = if is_cuse {
		crate::ROOT_ID
	} else {
		decode::node_id(buf.header().nodeid)?
	};
	let mut dec = decode::RequestDecoder::new(buf);

	// FUSE v7.8 added new fields to `fuse_release_in`.
	if version_minor < 8 {
		let raw: &'a fuse_release_in_v7p1 = dec.next_sized()?;
		return Ok(ReleaseRequest {
			phantom: PhantomData,
			node_id,
			handle: raw.fh,
			lock_owner: None,
			open_flags: raw.flags,
		});
	}

	let raw: &'a fuse_kernel::fuse_release_in = dec.next_sized()?;

	let mut lock_owner = None;
	if raw.release_flags & fuse_kernel::FUSE_RELEASE_FLOCK_UNLOCK != 0 {
		lock_owner = Some(raw.lock_owner);
	}

	Ok(ReleaseRequest {
		phantom: PhantomData,
		node_id,
		handle: raw.fh,
		lock_owner,
		open_flags: raw.flags,
	})
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
