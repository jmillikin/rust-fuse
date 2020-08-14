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

#![cfg_attr(doc, doc(cfg(feature = "unstable")))]

use crate::protocol::prelude::*;

// UnknownRequest {{{

pub struct UnknownRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a [u8],
}

impl<'a> UnknownRequest<'a> {
	pub fn node_id(&self) -> Option<NodeId> {
		NodeId::new(self.header.nodeid)
	}

	pub fn request_id(&self) -> u64 {
		self.header.unique
	}

	pub fn user_id(&self) -> u32 {
		self.header.uid
	}

	pub fn group_id(&self) -> u32 {
		self.header.gid
	}

	pub fn process_id(&self) -> u32 {
		self.header.pid
	}

	pub fn body(&self) -> &'a [u8] {
		self.body
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for UnknownRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		let body_offset = size_of::<fuse_kernel::fuse_in_header>() as u32;
		let body = dec.next_bytes(header.len - body_offset)?;
		Ok(Self { header, body })
	}
}

// }}}
