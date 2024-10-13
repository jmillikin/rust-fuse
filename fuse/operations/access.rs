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
use core::marker::PhantomData;

use crate::kernel;
use crate::server::decode;

// AccessRequest {{{

/// Request type for `FUSE_ACCESS`.
#[derive(Clone, Copy)]
pub struct AccessRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: crate::NodeId,
	mask: u32,
}

impl AccessRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}

	#[must_use]
	pub fn mask(&self) -> u32 {
		self.mask
	}
}

try_from_fuse_request!(AccessRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_ACCESS)?;
	let raw: &'a kernel::fuse_access_in = dec.next_sized()?;
	Ok(Self {
		phantom: PhantomData,
		node_id: decode::node_id(dec.header().nodeid)?,
		mask: raw.mask,
	})
});

impl fmt::Debug for AccessRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("AccessRequest")
			.field("node_id", &self.node_id)
			.field("mask", &self.mask)
			.finish()
	}
}

// }}}
