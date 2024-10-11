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
use crate::server;
use crate::server::decode;

// OpenRequest {{{

/// Request type for `FUSE_OPEN`.
pub struct OpenRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_open_in,
}

impl OpenRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		crate::NodeId::new(self.header.nodeid).unwrap_or(crate::NodeId::ROOT)
	}

	#[must_use]
	pub fn flags(&self) -> OpenRequestFlags {
		OpenRequestFlags {
			bits: self.body.open_flags,
		}
	}

	#[must_use]
	#[allow(clippy::misnamed_getters)]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.flags
	}
}

try_from_cuse_request!(OpenRequest<'a>, |request| {
	Self::try_from(request.inner, true)
});

try_from_fuse_request!(OpenRequest<'a>, |request| {
	Self::try_from(request.inner, false)
});

impl<'a> OpenRequest<'a> {
	fn try_from(
		request: server::Request<'a>,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_OPEN)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}
		Ok(Self { header, body })
	}
}

impl fmt::Debug for OpenRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenRequest")
			.field("node_id", &self.node_id())
			.field("flags", &self.flags())
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}

// OpenRequestFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::kernel;
	bitflags!(OpenRequestFlag, OpenRequestFlags, u32, {
		KILL_SUIDGID = kernel::FUSE_OPEN_KILL_SUIDGID;
	});
}

// }}}

// OpenResponseFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenResponseFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenResponseFlag {
	mask: u32,
}

mod response_flags {
	use crate::kernel;
	bitflags!(OpenResponseFlag, OpenResponseFlags, u32, {
		DIRECT_IO = kernel::FOPEN_DIRECT_IO;
		KEEP_CACHE = kernel::FOPEN_KEEP_CACHE;
		NONSEEKABLE = kernel::FOPEN_NONSEEKABLE;
		CACHE_DIR = kernel::FOPEN_CACHE_DIR;
		STREAM = kernel::FOPEN_STREAM;
		NOFLUSH = kernel::FOPEN_NOFLUSH;
	});
}

// }}}
