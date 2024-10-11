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

// OpendirRequest {{{

/// Request type for `FUSE_OPENDIR`.
pub struct OpendirRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_open_in,
}

impl OpendirRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn flags(&self) -> OpendirRequestFlags {
		OpendirRequestFlags {
			bits: self.body.open_flags,
		}
	}

	#[must_use]
	#[allow(clippy::misnamed_getters)]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.flags
	}
}

try_from_fuse_request!(OpendirRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_OPENDIR)?;

	let header = dec.header();
	let body = dec.next_sized()?;
	decode::node_id(header.nodeid)?;
	Ok(Self { header, body })
});

impl fmt::Debug for OpendirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpendirRequest")
			.field("node_id", &self.node_id())
			.field("flags", &self.flags())
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}

// OpendirRequestFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::kernel;
	bitflags!(OpendirRequestFlag, OpendirRequestFlags, u32, {
		KILL_SUIDGID = kernel::FUSE_OPEN_KILL_SUIDGID;
	});
}

// }}}

// OpendirResponseFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirResponseFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpendirResponseFlag {
	mask: u32,
}

mod response_flags {
	use crate::kernel;
	bitflags!(OpendirResponseFlag, OpendirResponseFlags, u32, {
		DIRECT_IO = kernel::FOPEN_DIRECT_IO;
		KEEP_CACHE = kernel::FOPEN_KEEP_CACHE;
		NONSEEKABLE = kernel::FOPEN_NONSEEKABLE;
		CACHE_DIR = kernel::FOPEN_CACHE_DIR;
		STREAM = kernel::FOPEN_STREAM;
		NOFLUSH = kernel::FOPEN_NOFLUSH;
	});
}

// }}}
