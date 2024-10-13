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

use crate::kernel;
use crate::server::decode;

// LseekRequest {{{

/// Request type for `FUSE_LSEEK`.
#[derive(Clone, Copy)]
pub struct LseekRequest<'a> {
	raw: &'a kernel::fuse_lseek_in,
	node_id: crate::NodeId,
}

impl LseekRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	#[must_use]
	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	#[must_use]
	pub fn whence(&self) -> LseekWhence {
		LseekWhence(self.raw.whence)
	}
}

try_from_fuse_request!(LseekRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_LSEEK)?;
	let raw = dec.next_sized()?;
	Ok(Self {
		raw,
		node_id: decode::node_id(dec.header().nodeid)?,
	})
});

impl fmt::Debug for LseekRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LseekRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("offset", &self.raw.offset)
			.field("whence", &LseekWhence(self.raw.whence))
			.finish()
	}
}

#[derive(Eq, PartialEq)]
pub struct LseekWhence(u32);

impl LseekWhence {
	pub const SEEK_DATA: LseekWhence = LseekWhence(3);
	pub const SEEK_HOLE: LseekWhence = LseekWhence(4);
}

impl fmt::Debug for LseekWhence {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self.0 {
			3 => fmt.write_str("SEEK_DATA"),
			4 => fmt.write_str("SEEK_HOLE"),
			_ => self.0.fmt(fmt),
		}
	}
}

// }}}
