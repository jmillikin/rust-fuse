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

use crate::FileMode;
use crate::Node;
use crate::NodeId;
use crate::NodeName;
use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;

// CreateRequest {{{

/// Request type for `FUSE_CREATE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_CREATE` operation.
pub struct CreateRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_create_in<'a>>,
	name: &'a NodeName,
}

impl<'a> CreateRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
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

		let name = NodeName::new(dec.next_nul_terminated_bytes()?);

		Ok(Self { header, body, name })
	}

	#[must_use]
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn name(&self) -> &NodeName {
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
	pub fn mode(&self) -> FileMode {
		if let Some(body) = self.body.as_v7p12() {
			return FileMode(body.mode);
		}
		FileMode(0)
	}

	#[must_use]
	pub fn umask(&self) -> u32 {
		if let Some(body) = self.body.as_v7p12() {
			return body.umask;
		}
		0
	}
}

impl fmt::Debug for CreateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CreateRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.field("flags", &self.flags())
			.field("open_flags", &DebugHexU32(self.open_flags()))
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
	entry_out: fuse_kernel::fuse_entry_out,
	handle: u64,
	flags: CreateResponseFlags,
}

impl<'a> CreateResponse<'a> {
	#[must_use]
	pub fn new() -> CreateResponse<'a> {
		Self {
			phantom: PhantomData,
			entry_out: fuse_kernel::fuse_entry_out::zeroed(),
			handle: 0,
			flags: CreateResponseFlags::new(),
		}
	}

	#[must_use]
	pub fn node(&self) -> &Node {
		Node::new_ref(&self.entry_out)
	}

	#[must_use]
	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.entry_out)
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.handle = handle;
	}

	#[must_use]
	pub fn flags(&self) -> CreateResponseFlags {
		self.flags
	}

	#[must_use]
	pub fn mut_flags(&mut self) -> &mut CreateResponseFlags {
		&mut self.flags
	}

	pub fn set_flags(&mut self, flags: CreateResponseFlags) {
		self.flags = flags;
	}
}

response_send_funcs!(CreateResponse<'_>);

impl fmt::Debug for CreateResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CreateResponse")
			.field("node", &self.entry_out)
			.field("handle", &self.handle)
			.field("flags", &self.flags)
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
		let open_out = fuse_kernel::fuse_open_out {
			fh: self.handle,
			open_flags: self.flags.bits,
			padding: 0,
		};
		self.node().encode_entry_sized(enc, ctx.version_minor, &open_out)
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
