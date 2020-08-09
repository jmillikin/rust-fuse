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
use crate::protocol::release::fuse_release_in_v7p1;

#[cfg(test)]
mod releasedir_test;

// ReleasedirRequest {{{

/// **\[UNSTABLE\]**
pub struct ReleasedirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: fuse_kernel::fuse_release_in,
}

impl ReleasedirRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn flags(&self) -> u32 {
		self.raw.flags
	}

	pub fn lock_owner(&self) -> Option<u64> {
		if self.raw.release_flags & fuse_kernel::FUSE_RELEASE_FLOCK_UNLOCK == 0
		{
			return None;
		}
		Some(self.raw.lock_owner)
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ReleasedirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_RELEASEDIR);

		// FUSE v7.8 added new fields to `fuse_release_in`.
		if dec.version().minor() < 8 {
			let raw: &'a fuse_release_in_v7p1 = dec.next_sized()?;
			return Ok(Self {
				header,
				raw: fuse_kernel::fuse_release_in {
					fh: raw.fh,
					flags: raw.flags,
					release_flags: 0,
					lock_owner: 0,
				},
			});
		}

		let raw: &'a fuse_kernel::fuse_release_in = dec.next_sized()?;
		Ok(Self { header, raw: *raw })
	}
}

// }}}

// ReleasedirResponse {{{

/// **\[UNSTABLE\]**
pub struct ReleasedirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl ReleasedirResponse<'_> {
	pub fn new() -> Self {
		ReleasedirResponse {
			phantom: PhantomData,
		}
	}
}

impl fuse_io::EncodeResponse for ReleasedirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_header_only()
	}
}

// }}}
