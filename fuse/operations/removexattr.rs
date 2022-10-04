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

//! Implements the `FUSE_REMOVEXATTR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;
use crate::xattr;

// RemovexattrRequest {{{

/// Request type for `FUSE_REMOVEXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_REMOVEXATTR` operation.
pub struct RemovexattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	name: &'a xattr::Name,
}

impl RemovexattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &xattr::Name {
		self.name
	}
}

request_try_from! { RemovexattrRequest : fuse }

impl decode::Sealed for RemovexattrRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for RemovexattrRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_REMOVEXATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let name_bytes = dec.next_nul_terminated_bytes()?;
		let name = xattr::Name::from_bytes(name_bytes.to_bytes_without_nul())?;
		Ok(Self { header, name })
	}
}

impl fmt::Debug for RemovexattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.finish()
	}
}

// }}}

// RemovexattrResponse {{{

/// Response type for `FUSE_REMOVEXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_REMOVEXATTR` operation.
pub struct RemovexattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> RemovexattrResponse<'a> {
	#[inline]
	#[must_use]
	pub fn new() -> RemovexattrResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(RemovexattrResponse<'_>);

impl fmt::Debug for RemovexattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrResponse").finish()
	}
}

impl RemovexattrResponse<'_> {
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
