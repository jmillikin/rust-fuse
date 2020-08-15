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
mod rename_test;

// RenameRequest {{{

const RENAME_NOREPLACE: u32 = 1 << 0;
const RENAME_EXCHANGE: u32 = 1 << 1;

pub struct RenameRequest<'a> {
	flags: u32,
	old_dir: NodeId,
	old_name: &'a CStr,
	new_dir: NodeId,
	new_name: &'a CStr,
}

impl RenameRequest<'_> {
	pub fn old_dir(&self) -> NodeId {
		self.old_dir
	}

	pub fn old_name(&self) -> &CStr {
		self.old_name
	}

	pub fn new_dir(&self) -> NodeId {
		self.new_dir
	}

	pub fn new_name(&self) -> &CStr {
		self.new_name
	}

	pub fn exchange(&self) -> bool {
		self.flags & RENAME_EXCHANGE > 0
	}

	pub fn no_replace(&self) -> bool {
		self.flags & RENAME_NOREPLACE > 0
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for RenameRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();

		let mut flags = 0;
		let new_dir: u64;
		if header.opcode == fuse_kernel::FUSE_RENAME2 {
			let parsed: &fuse_kernel::fuse_rename2_in = dec.next_sized()?;
			flags = parsed.flags;
			new_dir = parsed.newdir;
		} else {
			debug_assert!(header.opcode == fuse_kernel::FUSE_RENAME);
			let parsed: &fuse_kernel::fuse_rename_in = dec.next_sized()?;
			new_dir = parsed.newdir;
		}
		let old_name = dec.next_cstr()?;
		let new_name = dec.next_cstr()?;
		Ok(Self {
			flags,
			old_dir: try_node_id(header.nodeid)?,
			old_name,
			new_dir: try_node_id(new_dir)?,
			new_name,
		})
	}
}

// }}}

// RenameResponse {{{

pub struct RenameResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> RenameResponse<'a> {
	pub fn new() -> RenameResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for RenameResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameResponse").finish()
	}
}

impl fuse_io::EncodeResponse for RenameResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Error> {
		enc.encode_header_only()
	}
}

// }}}
