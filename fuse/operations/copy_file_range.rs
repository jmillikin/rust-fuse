// Copyright 2022 John Millikin and the rust-fuse contributors.
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

// CopyFileRangeRequest {{{

/// Request type for `FUSE_COPY_FILE_RANGE`.
pub struct CopyFileRangeRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_copy_file_range_in,
}

impl CopyFileRangeRequest<'_> {
	#[must_use]
	pub fn input_node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn input_handle(&self) -> u64 {
		self.body.fh_in
	}

	#[must_use]
	pub fn input_offset(&self) -> u64 {
		self.body.off_in
	}

	#[must_use]
	pub fn output_node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.body.nodeid_out) }
	}

	#[must_use]
	pub fn output_handle(&self) -> u64 {
		self.body.fh_out
	}

	#[must_use]
	pub fn output_offset(&self) -> u64 {
		self.body.off_out
	}

	#[must_use]
	pub fn len(&self) -> u64 {
		self.body.len
	}

	#[must_use]
	pub fn flags(&self) -> CopyFileRangeRequestFlags {
		CopyFileRangeRequestFlags {
			bits: self.body.flags,
		}
	}
}

try_from_fuse_request!(CopyFileRangeRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_COPY_FILE_RANGE)?;

	use kernel::fuse_copy_file_range_in;

	let header = dec.header();
	let body: &'a fuse_copy_file_range_in = dec.next_sized()?;
	decode::node_id(header.nodeid)?;
	decode::node_id(body.nodeid_out)?;

	Ok(Self { header, body })
});

impl fmt::Debug for CopyFileRangeRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CopyFileRangeRequest")
			.field("input_node_id", &self.input_node_id())
			.field("input_handle", &self.input_handle())
			.field("input_offset", &self.input_offset())
			.field("output_node_id", &self.output_node_id())
			.field("output_handle", &self.output_handle())
			.field("output_offset", &self.output_offset())
			.field("len", &self.len())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// CopyFileRangeRequestFlags {{{

/// Optional flags set on [`CopyFileRangeRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CopyFileRangeRequestFlags {
	bits: u64,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CopyFileRangeRequestFlag {
	mask: u64,
}

mod request_flags {
	bitflags!(CopyFileRangeRequestFlag, CopyFileRangeRequestFlags, u64, {
	});
}

// }}}
