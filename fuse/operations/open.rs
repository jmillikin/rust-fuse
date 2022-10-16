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

//! Implements the `FUSE_OPEN` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// OpenRequest {{{

/// Request type for `FUSE_OPEN`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPEN` operation.
pub struct OpenRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_open_in,
}

impl OpenRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		node::Id::new(self.header.nodeid).unwrap_or(node::Id::ROOT)
	}

	#[must_use]
	pub fn flags(&self) -> OpenRequestFlags {
		OpenRequestFlags {
			bits: self.body.open_flags,
		}
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.flags
	}
}

impl server::sealed::Sealed for OpenRequest<'_> {}

impl<'a> server::CuseRequest<'a> for OpenRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::CuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request, true)
	}
}

impl<'a> server::FuseRequest<'a> for OpenRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request, false)
	}
}

impl<'a> OpenRequest<'a> {
	fn decode_request(
		request: server::Request<'a>,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_OPEN)?;

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

// OpenResponse {{{

/// Response type for `FUSE_OPEN`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_OPEN` operation.
pub struct OpenResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_open_out,
}

impl<'a> OpenResponse<'a> {
	#[must_use]
	pub fn new() -> OpenResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_open_out::zeroed(),
		}
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.raw.fh = handle;
	}

	#[must_use]
	pub fn flags(&self) -> OpenResponseFlags {
		OpenResponseFlags {
			bits: self.raw.open_flags,
		}
	}

	pub fn set_flags(&mut self, flags: OpenResponseFlags) {
		self.raw.open_flags = flags.bits
	}

	#[inline]
	pub fn update_flags(&mut self, f: impl FnOnce(&mut OpenResponseFlags)) {
		let mut flags = self.flags();
		f(&mut flags);
		self.set_flags(flags)
	}
}

impl fmt::Debug for OpenResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenResponse")
			.field("handle", &self.handle())
			.field("flags", &self.flags())
			.finish()
	}
}

impl server::sealed::Sealed for OpenResponse<'_> {}

impl server::CuseResponse for OpenResponse<'_> {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::CuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
	}
}

impl server::FuseResponse for OpenResponse<'_> {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
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
	use crate::internal::fuse_kernel;
	bitflags!(OpenRequestFlag, OpenRequestFlags, u32, {
		KILL_SUIDGID = fuse_kernel::FUSE_OPEN_KILL_SUIDGID;
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
	use crate::internal::fuse_kernel;
	bitflags!(OpenResponseFlag, OpenResponseFlags, u32, {
		DIRECT_IO = fuse_kernel::FOPEN_DIRECT_IO;
		KEEP_CACHE = fuse_kernel::FOPEN_KEEP_CACHE;
		NONSEEKABLE = fuse_kernel::FOPEN_NONSEEKABLE;
		CACHE_DIR = fuse_kernel::FOPEN_CACHE_DIR;
		STREAM = fuse_kernel::FOPEN_STREAM;
		NOFLUSH = fuse_kernel::FOPEN_NOFLUSH;
	});
}

// }}}
