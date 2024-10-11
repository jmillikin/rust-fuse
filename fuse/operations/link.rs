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

use crate::kernel;
use crate::server::decode;

// LinkRequest {{{

/// Request type for `FUSE_LINK`.
#[derive(Debug)]
pub struct LinkRequest<'a> {
	node_id: crate::NodeId,
	new_parent_id: crate::NodeId,
	new_name: &'a crate::NodeName,
}

impl LinkRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}

	#[must_use]
	pub fn new_parent_id(&self) -> crate::NodeId {
		self.new_parent_id
	}

	#[must_use]
	pub fn new_name(&self) -> &crate::NodeName {
		self.new_name
	}
}

try_from_fuse_request!(LinkRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_LINK)?;

	let raw: &kernel::fuse_link_in = dec.next_sized()?;
	let name = dec.next_node_name()?;
	Ok(Self {
		node_id: decode::node_id(raw.oldnodeid)?,
		new_parent_id: decode::node_id(dec.header().nodeid)?,
		new_name: name,
	})
});

// }}}
