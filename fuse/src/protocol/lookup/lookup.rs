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
mod lookup_test;

// LookupRequest {{{

/// **\[UNSTABLE\]** Request type for [`FuseHandlers::lookup`].
///
/// [`FuseHandlers::lookup`]: ../trait.FuseHandlers.html#method.lookup
#[derive(Debug)]
pub struct LookupRequest<'a> {
	node_id: node::NodeId,
	name: &'a CStr,
}

impl LookupRequest<'_> {
	pub fn node_id(&self) -> node::NodeId {
		self.node_id
	}

	pub fn name(&self) -> &CStr {
		&self.name
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for LookupRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_LOOKUP);

		Ok(Self {
			node_id: try_node_id(header.nodeid)?,
			name: dec.next_cstr()?,
		})
	}
}

// }}}

// LookupResponse {{{

/// **\[UNSTABLE\]** Response type for [`FuseHandlers::lookup`].
///
/// [`FuseHandlers::lookup`]: ../trait.FuseHandlers.html#method.lookup
pub struct LookupResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl LookupResponse<'_> {
	pub fn new() -> Self {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_entry_out {
				nodeid: 0,
				generation: 0,
				entry_valid: 0,
				attr_valid: 0,
				entry_valid_nsec: 0,
				attr_valid_nsec: 0,
				attr: Default::default(),
			},
		}
	}

	entry_out_methods!(raw);
}

impl fmt::Debug for LookupResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LookupResponse")
			.field("node_id", &self.node_id())
			.field("cache_duration", &self.cache_duration())
			.field("attr", self.attr())
			.field("attr_cache_duration", &self.attr_cache_duration())
			.finish()
	}
}

impl fuse_io::EncodeResponse for LookupResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		// In early versions of FUSE, `fuse_entry_out::nodeid` was a required
		// field and must be non-zero. FUSE v7.4 relaxed this so that a zero
		// node ID was the same as returning ENOENT, but with a cache hint.
		if self.raw.nodeid == 0 && enc.version().minor() < 4 {
			return enc.encode_error(-libc::ENOENT);
		}

		crate::protocol::common::encode_entry_out(enc, &self.raw)
	}
}

// }}}
