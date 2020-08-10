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

use crate::protocol::node;
use crate::protocol::prelude::*;

#[cfg(test)]
mod open_test;

// OpenRequest {{{

/// Request type for [`FuseHandlers::open`].
///
/// [`FuseHandlers::open`]: ../trait.FuseHandlers.html#method.open
pub struct OpenRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: node::NodeId,
	flags: u32,
}

impl OpenRequest<'_> {
	pub fn node_id(&self) -> node::NodeId {
		self.node_id
	}

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

impl<'a> fuse_io::DecodeRequest<'a> for OpenRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_OPEN);

		let raw: &'a fuse_kernel::fuse_open_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
			flags: raw.flags,
		})
	}
}

// }}}

// OpenResponse {{{

/// Response type for [`FuseHandlers::open`].
///
/// [`FuseHandlers::open`]: ../trait.FuseHandlers.html#method.open
pub struct OpenResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_open_out,
}

impl OpenResponse<'_> {
	pub fn new() -> Self {
		OpenResponse {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_open_out {
				fh: 0,
				open_flags: 0,
				padding: 0,
			},
		}
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.raw.fh = handle;
	}

	pub fn flags(&self) -> OpenFlags {
		OpenFlags(self.raw.open_flags)
	}

	pub fn set_flags(&mut self, flags: OpenFlags) {
		self.raw.open_flags = flags.0;
	}
}

impl fmt::Debug for OpenResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenResponse")
			.field("handle", &self.raw.fh)
			.field("flags", &self.flags())
			.finish()
	}
}

impl fuse_io::EncodeResponse for OpenResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_sized(&self.raw)
	}
}

// }}}

// OpenFlags {{{

bitflags_struct! {
	pub struct OpenFlags(u32);

	FOPEN_DIRECT_IO: {
		get: direct_io,
		set: set_direct_io,
	},
	FOPEN_KEEP_CACHE: {
		get: keep_cache,
		set: set_keep_cache,
	},
	FOPEN_NONSEEKABLE: {
		get: nonseekable,
		set: set_nonseekable,
	},
}

// }}}
