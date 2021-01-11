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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "create_test")]
mod create_test;

// CreateRequest {{{

/// Request type for [`FuseHandlers::create`].
///
/// [`FuseHandlers::create`]: ../../trait.FuseHandlers.html#method.create
pub struct CreateRequest<'a> {
	node_id: NodeId,
	name: &'a NodeName,
	flags: u32,
	mode: u32,
	umask: u32,
}

impl CreateRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn name(&self) -> &NodeName {
		self.name
	}

	pub fn flags(&self) -> u32 {
		self.flags
	}

	pub fn mode(&self) -> FileMode {
		FileMode(self.mode)
	}

	pub fn umask(&self) -> u32 {
		self.umask
	}
}

impl fmt::Debug for CreateRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CreateRequest")
			.field("node_id", &self.node_id)
			.field("name", &self.name)
			.field("flags", &self.flags)
			.field("mode", &FileMode(self.mode))
			.field("umask", &self.umask)
			.finish()
	}
}

#[repr(C)]
struct fuse_create_in_v7p1 {
	pub flags: u32,
	pub unused: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for CreateRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_CREATE);

		let node_id = try_node_id(header.nodeid)?;

		if dec.version().minor() < 12 {
			let raw: &'a fuse_create_in_v7p1 = dec.next_sized()?;
			let name = NodeName::new(dec.next_nul_terminated_bytes()?);
			return Ok(Self {
				node_id,
				name,
				flags: raw.flags,
				mode: 0,
				umask: 0,
			});
		}

		let raw: &'a fuse_kernel::fuse_create_in = dec.next_sized()?;
		let name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			node_id,
			name,
			flags: raw.flags,
			mode: raw.mode,
			umask: raw.umask,
		})
	}
}

// }}}

// CreateResponse {{{

/// Response type for [`FuseHandlers::create`].
///
/// [`FuseHandlers::create`]: ../../trait.FuseHandlers.html#method.create
pub struct CreateResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry_out: fuse_kernel::fuse_entry_out,
	handle: u64,
	flags: CreateResponseFlags,
}

impl<'a> CreateResponse<'a> {
	pub fn new() -> CreateResponse<'a> {
		Self {
			phantom: PhantomData,
			entry_out: Default::default(),
			handle: 0,
			flags: CreateResponseFlags::new(),
		}
	}

	pub fn node(&self) -> &Node {
		Node::new_ref(&self.entry_out)
	}

	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.entry_out)
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.handle = handle;
	}

	pub fn flags(&self) -> &CreateResponseFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut CreateResponseFlags {
		&mut self.flags
	}
}

impl fmt::Debug for CreateResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CreateResponse")
			.field("node", &self.entry_out)
			.field("handle", &self.handle)
			.field("flags", &self.flags)
			.finish()
	}
}

impl fuse_io::EncodeResponse for CreateResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		let open_out = fuse_kernel::fuse_open_out {
			fh: self.handle,
			open_flags: self.flags.to_bits(),
			padding: 0,
		};
		self.node().encode_entry_sized(enc, &open_out)
	}
}

// }}}

// CreateResponseFlags {{{

bitflags_struct! {
	/// Optional flags set on [`CreateResponse`].
	///
	/// [`CreateResponse`]: struct.CreateResponse.html
	pub struct CreateResponseFlags(u32);

	/// Use [page-based direct I/O][direct-io] on this file.
	///
	/// [direct-io]: https://lwn.net/Articles/348719/
	fuse_kernel::FOPEN_DIRECT_IO: direct_io,

	/// Allow the kernel to preserve cached file data from the last time this
	/// file was opened.
	fuse_kernel::FOPEN_KEEP_CACHE: keep_cache,

	/// Tell the kernel this file is not seekable.
	fuse_kernel::FOPEN_NONSEEKABLE: nonseekable,
}

// }}}
