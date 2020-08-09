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

#[cfg(test)]
mod removexattr_test;

// RemovexattrRequest {{{

/// **\[UNSTABLE\]**
pub struct RemovexattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	name: &'a CStr,
}

impl RemovexattrRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn name(&self) -> &CStr {
		self.name
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for RemovexattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_REMOVEXATTR);

		let name = dec.next_cstr()?;
		Ok(Self { header, name })
	}
}

// }}}

// RemovexattrResponse {{{

/// **\[UNSTABLE\]**
pub struct RemovexattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl RemovexattrResponse<'_> {
	pub fn new() -> Self {
		RemovexattrResponse {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for RemovexattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrResponse").finish()
	}
}

impl fuse_io::EncodeResponse for RemovexattrResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_header_only()
	}
}

// }}}
