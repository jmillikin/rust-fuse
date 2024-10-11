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

use core::fmt;

use crate::internal::debug;
use crate::kernel;
use crate::server::decode;

// RenameRequest {{{

/// Request type for `FUSE_RENAME` and `FUSE_RENAME2`.
pub struct RenameRequest<'a> {
	old_directory_id: crate::NodeId,
	old_name: &'a crate::NodeName,
	new_directory_id: crate::NodeId,
	new_name: &'a crate::NodeName,
	rename_flags: u32,
}

impl RenameRequest<'_> {
	#[must_use]
	pub fn old_directory_id(&self) -> crate::NodeId {
		self.old_directory_id
	}

	#[must_use]
	pub fn old_name(&self) -> &crate::NodeName {
		self.old_name
	}

	#[must_use]
	pub fn new_directory_id(&self) -> crate::NodeId {
		self.new_directory_id
	}

	#[must_use]
	pub fn new_name(&self) -> &crate::NodeName {
		self.new_name
	}

	#[must_use]
	pub fn rename_flags(&self) -> crate::RenameFlags {
		self.rename_flags
	}
}

try_from_fuse_request!(RenameRequest<'a>, |request| {
	let mut dec = request.decoder();
	let header = dec.header();

	let mut rename_flags = 0;
	let new_dir: u64;
	if header.opcode == kernel::fuse_opcode::FUSE_RENAME2 {
		let parsed: &kernel::fuse_rename2_in = dec.next_sized()?;
		rename_flags = parsed.flags;
		new_dir = parsed.newdir;
	} else {
		dec.expect_opcode(kernel::fuse_opcode::FUSE_RENAME)?;
		let parsed: &kernel::fuse_rename_in = dec.next_sized()?;
		new_dir = parsed.newdir;
	}
	let old_name = dec.next_node_name()?;
	let new_name = dec.next_node_name()?;
	Ok(Self {
		old_directory_id: decode::node_id(header.nodeid)?,
		old_name,
		new_directory_id: decode::node_id(new_dir)?,
		new_name,
		rename_flags,
	})
});

impl fmt::Debug for RenameRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameRequest")
			.field("old_directory_id", &self.old_directory_id)
			.field("old_name", &self.old_name)
			.field("new_directory_id", &self.new_directory_id)
			.field("new_name", &self.new_name)
			.field("rename_flags", &debug::hex_u32(self.rename_flags))
			.finish()
	}
}

// }}}
