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

// OpenRequest {{{

/// Request type for [`FuseHandlers::open`].
///
/// [`FuseHandlers::open`]: ../../trait.FuseHandlers.html#method.open
pub struct OpenRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	flags: u32,
}

impl OpenRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	/// Platform-specific flags passed to [`open(2)`].
	///
	/// [`open(2)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/open.html
	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl fmt::Debug for OpenRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenRequest")
			.field("node_id", &self.node_id)
			.field("flags", &DebugHexU32(self.flags))
			.finish()
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for OpenRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_OPEN);

		let node_id = if dec.is_cuse() {
			crate::ROOT_ID
		} else {
			try_node_id(header.nodeid)?
		};

		let raw: &'a fuse_kernel::fuse_open_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id,
			flags: raw.flags,
		})
	}
}

// }}}

// OpenResponse {{{

/// Response type for [`FuseHandlers::open`].
///
/// [`FuseHandlers::open`]: ../../trait.FuseHandlers.html#method.open
pub struct OpenResponse<'a> {
	phantom: PhantomData<&'a ()>,
	handle: u64,
	flags: OpenResponseFlags,
}

impl<'a> OpenResponse<'a> {
	pub fn new() -> OpenResponse<'a> {
		Self {
			phantom: PhantomData,
			handle: 0,
			flags: OpenResponseFlags::new(),
		}
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn set_handle(&mut self, handle: u64) {
		self.handle = handle;
	}

	pub fn flags(&self) -> &OpenResponseFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut OpenResponseFlags {
		&mut self.flags
	}
}

impl fmt::Debug for OpenResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("OpenResponse")
			.field("handle", &self.handle)
			.field("flags", &self.flags())
			.finish()
	}
}

impl fuse_io::EncodeResponse for OpenResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_sized(&fuse_kernel::fuse_open_out {
			fh: self.handle,
			open_flags: self.flags.to_bits(),
			padding: 0,
		})
	}
}

// }}}

// OpenResponseFlags {{{

bitflags_struct! {
	/// Optional flags set on [`OpenResponse`].
	///
	/// [`OpenResponse`]: struct.OpenResponse.html
	pub struct OpenResponseFlags(u32);

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
