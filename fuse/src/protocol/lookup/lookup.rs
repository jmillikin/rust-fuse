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

#[cfg(rust_fuse_test = "lookup_test")]
mod lookup_test;

// LookupRequest {{{

/// Request type for [`FuseHandlers::lookup`].
///
/// [`FuseHandlers::lookup`]: ../../trait.FuseHandlers.html#method.lookup
#[derive(Debug)]
pub struct LookupRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
}

impl LookupRequest<'_> {
	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	pub fn name(&self) -> &NodeName {
		self.name
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for LookupRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::RequestError> {
		buf.expect_opcode(fuse_kernel::FUSE_LOOKUP)?;
		let mut dec = decode::RequestDecoder::new(buf);
		Ok(Self {
			parent_id: try_node_id(buf.header().nodeid)?,
			name: NodeName::new(dec.next_nul_terminated_bytes()?),
		})
	}
}

// }}}

// LookupResponse {{{

/// Response type for [`FuseHandlers::lookup`].
///
/// [`FuseHandlers::lookup`]: ../../trait.FuseHandlers.html#method.lookup
pub struct LookupResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry_out: fuse_kernel::fuse_entry_out,
}

impl<'a> LookupResponse<'a> {
	pub fn new() -> LookupResponse<'a> {
		Self {
			phantom: PhantomData,
			entry_out: fuse_kernel::fuse_entry_out::zeroed(),
		}
	}

	pub fn node(&self) -> &Node {
		Node::new_ref(&self.entry_out)
	}

	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.entry_out)
	}
}

impl fmt::Debug for LookupResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LookupResponse")
			.field("node", self.node())
			.finish()
	}
}

impl encode::EncodeReply for LookupResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		// In early versions of FUSE, `fuse_entry_out::nodeid` was a required
		// field and must be non-zero. FUSE v7.4 relaxed this so that a zero
		// node ID was the same as returning ENOENT, but with a cache hint.
		if self.entry_out.nodeid == 0 && version_minor < 4 {
			return enc.encode_error(crate::Error::NOT_FOUND);
		}
		self.node().encode_entry(enc, version_minor)
	}
}

// }}}
