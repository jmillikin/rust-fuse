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

// FsyncdirRequest {{{

/// Request type for `FUSE_FSYNCDIR`.
#[derive(Clone, Copy)]
pub struct FsyncdirRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_fsync_in,
}

impl FsyncdirRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub fn flags(&self) -> FsyncdirRequestFlags {
		FsyncdirRequestFlags {
			bits: self.body.fsync_flags,
		}
	}
}

try_from_fuse_request!(FsyncdirRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_FSYNCDIR)?;

	let header = dec.header();
	let body = dec.next_sized()?;
	decode::node_id(header.nodeid)?;
	Ok(Self { header, body })
});

impl fmt::Debug for FsyncdirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// FsyncdirRequestFlags {{{

/// Optional flags set on [`FsyncdirRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncdirRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncdirRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::kernel;
	bitflags!(FsyncdirRequestFlag, FsyncdirRequestFlags, u32, {
		FDATASYNC = kernel::FUSE_FSYNC_FDATASYNC;
	});
}

// }}}
