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
mod statfs_test;

// StatfsRequest {{{

pub struct StatfsRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
}

impl StatfsRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for StatfsRequest<'a> {
	fn decode_request(dec: fuse_io::RequestDecoder<'a>) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_STATFS);
		Ok(Self { header })
	}
}

// }}}

// StatfsResponse {{{

pub struct StatfsResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_statfs_out,
}

impl<'a> StatfsResponse<'a> {
	pub fn new() -> StatfsResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: Default::default(),
		}
	}
}

impl fmt::Debug for StatfsResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("StatfsResponse").finish()
	}
}

impl fuse_io::EncodeResponse for StatfsResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Error> {
		enc.encode_sized(&self.raw)
	}
}

// }}}
