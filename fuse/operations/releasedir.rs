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

//! Implements the `FUSE_RELEASEDIR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;

// ReleasedirRequest {{{

/// Request type for `FUSE_RELEASEDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASEDIR` operation.
pub struct ReleasedirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_release_in<'a>>,
}

impl<'a> ReleasedirRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let version_minor = request.version_minor;
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_RELEASEDIR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = if version_minor >= 8 {
			let body_v7p8 = dec.next_sized()?;
			compat::Versioned::new_release_v7p8(version_minor, body_v7p8)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_release_v7p1(version_minor, body_v7p1)
		};

		Ok(Self { header, body })
	}

	#[must_use]
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	/// The value passed to [`OpendirResponse::set_handle`], or zero if not set.
	///
	/// [`OpendirResponse::set_handle`]: crate::operations::opendir::OpendirResponse::set_handle
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.as_v7p1().fh
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<u64> {
		let body = self.body.as_v7p8()?;
		if body.release_flags & fuse_kernel::FUSE_RELEASE_FLOCK_UNLOCK == 0 {
			return None;
		}
		Some(body.lock_owner)
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.as_v7p1().flags
	}
}

impl fmt::Debug for ReleasedirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleasedirRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("open_flags", &DebugHexU32(self.open_flags()))
			.finish()
	}
}

// }}}

// ReleasedirResponse {{{

/// Response type for `FUSE_RELEASEDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASEDIR` operation.
pub struct ReleasedirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> ReleasedirResponse<'a> {
	#[must_use]
	pub fn new() -> ReleasedirResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(ReleasedirResponse<'_>);

impl fmt::Debug for ReleasedirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleasedirResponse").finish()
	}
}

impl ReleasedirResponse<'_> {
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
