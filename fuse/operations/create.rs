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

//! Implements the `FUSE_CREATE` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::compat;
use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// CreateRequest {{{

/// Request type for `FUSE_CREATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_CREATE` operation.
pub struct CreateRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_create_in<'a>>,
	name: &'a node::Name,
}

impl CreateRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}

	#[must_use]
	pub fn flags(&self) -> CreateRequestFlags {
		if let Some(body) = self.body.as_v7p12() {
			return CreateRequestFlags {
				bits: body.open_flags,
			};
		}
		CreateRequestFlags::new()
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.as_v7p1().flags
	}

	#[must_use]
	pub fn mode(&self) -> node::Mode {
		if let Some(body) = self.body.as_v7p12() {
			return node::Mode::new(body.mode);
		}
		node::Mode::new(0)
	}

	#[must_use]
	pub fn umask(&self) -> u32 {
		if let Some(body) = self.body.as_v7p12() {
			return body.umask;
		}
		0
	}
}

impl decode::Sealed for CreateRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for CreateRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let version_minor = request.version_minor;
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_CREATE)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = if version_minor >= 12 {
			let body_v7p12 = dec.next_sized()?;
			compat::Versioned::new_create_v7p12(version_minor, body_v7p12)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_create_v7p1(version_minor, body_v7p1)
		};

		let name = dec.next_node_name()?;

		Ok(Self { header, body, name })
	}
}

request_try_from! { CreateRequest : fuse }

impl fmt::Debug for CreateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CreateRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.field("flags", &self.flags())
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.field("mode", &self.mode())
			.field("umask", &self.umask())
			.finish()
	}
}

// }}}

// CreateResponse {{{

/// Response type for `FUSE_CREATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_CREATE` operation.
pub struct CreateResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry: node::Entry,
	open_out: fuse_kernel::fuse_open_out,
}

impl<'a> CreateResponse<'a> {
	#[inline]
	#[must_use]
	pub fn new(entry: node::Entry) -> CreateResponse<'a> {
		Self {
			phantom: PhantomData,
			entry,
			open_out: fuse_kernel::fuse_open_out::zeroed(),
		}
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> &node::Entry {
		&self.entry
	}

	#[inline]
	#[must_use]
	pub fn entry_mut(&mut self) -> &mut node::Entry {
		&mut self.entry
	}

	#[inline]
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.open_out.fh
	}

	#[inline]
	pub fn set_handle(&mut self, handle: u64) {
		self.open_out.fh = handle;
	}

	#[inline]
	#[must_use]
	pub fn flags(&self) -> CreateResponseFlags {
		CreateResponseFlags { bits: self.open_out.open_flags }
	}

	#[inline]
	pub fn set_flags(&mut self, flags: CreateResponseFlags) {
		self.open_out.open_flags = flags.bits;
	}

	#[inline]
	pub fn update_flags(&mut self, f: impl FnOnce(&mut CreateResponseFlags)) {
		let mut flags = self.flags();
		f(&mut flags);
		self.set_flags(flags)
	}
}

response_send_funcs!(CreateResponse<'_>);

impl fmt::Debug for CreateResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CreateResponse")
			.field("entry", &self.entry())
			.field("handle", &self.handle())
			.field("flags", &self.flags())
			.finish()
	}
}

impl CreateResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		if ctx.version_minor >= 9 {
			return enc.encode_sized_sized(self.entry.as_v7p9(), &self.open_out)
		}
		enc.encode_sized_sized(self.entry.as_v7p1(), &self.open_out)
	}
}

// }}}

// CreateRequestFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CreateRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CreateRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(CreateRequestFlag, CreateRequestFlags, u32, {
		KILL_SUIDGID = fuse_kernel::FUSE_OPEN_KILL_SUIDGID;
	});
}

// }}}

// CreateResponseFlags {{{

/// Optional flags set on [`CreateResponse`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CreateResponseFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CreateResponseFlag {
	mask: u32,
}

mod response_flags {
	use crate::internal::fuse_kernel;
	bitflags!(CreateResponseFlag, CreateResponseFlags, u32, {
		DIRECT_IO = fuse_kernel::FOPEN_DIRECT_IO;
		KEEP_CACHE = fuse_kernel::FOPEN_KEEP_CACHE;
		NONSEEKABLE = fuse_kernel::FOPEN_NONSEEKABLE;
		CACHE_DIR = fuse_kernel::FOPEN_CACHE_DIR;
		STREAM = fuse_kernel::FOPEN_STREAM;
		NOFLUSH = fuse_kernel::FOPEN_NOFLUSH;
	});
}

// }}}
