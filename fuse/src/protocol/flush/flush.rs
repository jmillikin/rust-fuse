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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "flush_test")]
mod flush_test;

// FlushRequest {{{

pub struct FlushRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	handle: u64,
	lock_owner: u64,
}

impl FlushRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn lock_owner(&self) -> u64 {
		self.lock_owner
	}
}

impl fmt::Debug for FlushRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FlushRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.finish()
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for FlushRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_FLUSH);

		let raw: &fuse_kernel::fuse_flush_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
			handle: raw.fh,
			lock_owner: raw.lock_owner,
		})
	}
}

// }}}

// FlushResponse {{{

pub struct FlushResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FlushResponse<'a> {
	pub fn new() -> FlushResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for FlushResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FlushResponse").finish()
	}
}

impl fuse_io::EncodeResponse for FlushResponse<'_> {
	fn encode_response<'a, S: io::OutputStream>(
		&'a self,
		enc: fuse_io::ResponseEncoder<S>,
	) -> Result<(), S::Error> {
		enc.encode_header_only()
	}
}

// }}}
