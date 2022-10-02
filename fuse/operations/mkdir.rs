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

//! Implements the `FUSE_MKDIR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::FileMode;
use crate::Node;
use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

// MkdirRequest {{{

/// Request type for `FUSE_MKDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_MKDIR` operation.
pub struct MkdirRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
	raw: fuse_kernel::fuse_mkdir_in,
}

impl<'a> MkdirRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_MKDIR)?;

		let raw: &fuse_kernel::fuse_mkdir_in = dec.next_sized()?;
		let name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name,
			raw: *raw,
		})
	}

	#[must_use]
	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &NodeName {
		self.name
	}

	#[must_use]
	pub fn mode(&self) -> FileMode {
		FileMode(self.raw.mode)
	}

	#[must_use]
	pub fn umask(&self) -> u32 {
		self.raw.umask
	}
}

impl fmt::Debug for MkdirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MkdirRequest")
			.field("parent_id", &self.parent_id())
			.field("name", &self.name())
			.field("mode", &self.mode())
			.field("umask", &format_args!("{:#o}", &self.raw.umask))
			.finish()
	}
}

// }}}

// MkdirResponse {{{

/// Response type for `FUSE_MKDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_MKDIR` operation.
pub struct MkdirResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> MkdirResponse<'a> {
	#[must_use]
	pub fn new() -> MkdirResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_entry_out::zeroed(),
		}
	}

	#[must_use]
	pub fn node(&self) -> &Node {
		Node::new_ref(&self.raw)
	}

	#[must_use]
	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.raw)
	}
}

response_send_funcs!(MkdirResponse<'_>);

impl fmt::Debug for MkdirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MkdirResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl MkdirResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		self.node().encode_entry(enc, ctx.version_minor)
	}
}

// }}}
