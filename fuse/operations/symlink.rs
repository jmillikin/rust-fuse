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

//! Implements the `FUSE_SYMLINK` operation.

use core::fmt;

use crate::kernel;
use crate::server::decode;

// SymlinkRequest {{{

/// Request type for `FUSE_SYMLINK`.
#[derive(Clone, Copy)]
pub struct SymlinkRequest<'a> {
	parent_id: crate::NodeId,
	name: &'a crate::NodeName,
	content: &'a core::ffi::CStr,
}

impl SymlinkRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> crate::NodeId {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
		self.name
	}

	#[must_use]
	pub fn content(&self) -> &core::ffi::CStr {
		self.content
	}
}

try_from_fuse_request!(SymlinkRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_SYMLINK)?;
	let content = dec.next_cstr()?;
	let name = dec.next_node_name()?;
	Ok(Self {
		parent_id: decode::node_id(dec.header().nodeid)?,
		name,
		content,
	})
});

impl fmt::Debug for SymlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkRequest")
			.field("parent_id", &self.parent_id)
			.field("name", &self.name)
			.field("content", &self.content)
			.finish()
	}
}

// }}}
