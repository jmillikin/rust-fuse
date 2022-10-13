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

//! Implements the `FUSE_SYMLINK` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// SymlinkRequest {{{

/// Request type for `FUSE_SYMLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYMLINK` operation.
pub struct SymlinkRequest<'a> {
	parent_id: node::Id,
	name: &'a node::Name,
	content: &'a [u8],
}

impl SymlinkRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}

	#[must_use]
	pub fn content(&self) -> &[u8] {
		self.content
	}
}

request_try_from! { SymlinkRequest : fuse }

impl decode::Sealed for SymlinkRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for SymlinkRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_SYMLINK)?;
		let content = dec.next_nul_terminated_bytes()?.to_bytes_without_nul();
		let name = dec.next_node_name()?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name,
			content,
		})
	}
}

impl fmt::Debug for SymlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkRequest")
			.field("parent_id", &self.parent_id)
			.field("name", &self.name)
			.field("content", &debug::bytes(self.content))
			.finish()
	}
}

// }}}

// SymlinkResponse {{{

/// Response type for `FUSE_SYMLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYMLINK` operation.
pub struct SymlinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry: node::Entry,
}

impl<'a> SymlinkResponse<'a> {
	#[must_use]
	pub fn new(entry: node::Entry) -> SymlinkResponse<'a> {
		Self {
			phantom: PhantomData,
			entry,
		}
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> &node::Entry {
		&self.entry
	}

	#[inline]
	#[must_use]
	pub fn entry_mut(&mut self) -> &mut node::Entry {
		&mut self.entry
	}
}

response_send_funcs!(SymlinkResponse<'_>);

impl fmt::Debug for SymlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkResponse")
			.field("entry", &self.entry())
			.finish()
	}
}

impl SymlinkResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		if ctx.version_minor >= 9 {
			return enc.encode_sized(self.entry.as_v7p9());
		}
		enc.encode_sized(self.entry.as_v7p1())
	}
}

// }}}
