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

//! Implements the `FUSE_LINK` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// LinkRequest {{{

/// Request type for `FUSE_LINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LINK` operation.
#[derive(Debug)]
pub struct LinkRequest<'a> {
	node_id: node::Id,
	new_parent_id: node::Id,
	new_name: &'a node::Name,
}

impl LinkRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		self.node_id
	}

	#[must_use]
	pub fn new_parent_id(&self) -> node::Id {
		self.new_parent_id
	}

	#[must_use]
	pub fn new_name(&self) -> &node::Name {
		self.new_name
	}
}

request_try_from! { LinkRequest : fuse }

impl decode::Sealed for LinkRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for LinkRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LINK)?;

		let raw: &fuse_kernel::fuse_link_in = dec.next_sized()?;
		let name = dec.next_node_name()?;
		Ok(Self {
			node_id: decode::node_id(raw.oldnodeid)?,
			new_parent_id: decode::node_id(dec.header().nodeid)?,
			new_name: name,
		})
	}
}

// }}}

// LinkResponse {{{

/// Response type for `FUSE_LINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LINK` operation.
pub struct LinkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry: node::Entry,
}

impl<'a> LinkResponse<'a> {
	#[inline]
	#[must_use]
	pub fn new(entry: node::Entry) -> LinkResponse<'a> {
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

response_send_funcs!(LinkResponse<'_>);

impl fmt::Debug for LinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LinkResponse")
			.field("entry", &self.entry())
			.finish()
	}
}

impl LinkResponse<'_> {
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
