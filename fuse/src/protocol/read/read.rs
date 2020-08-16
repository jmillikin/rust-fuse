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

#[cfg(test)]
mod read_test;

// ReadRequest {{{

/// Request type for [`FuseHandlers::read`].
///
/// [`FuseHandlers::read`]: ../trait.FuseHandlers.html#method.read
pub struct ReadRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	size: u32,
	offset: u64,
	handle: u64,
	lock_owner: Option<u64>,
	open_flags: u32,
}

impl ReadRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	pub fn offset(&self) -> u64 {
		self.offset
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: struct.OpenResponse.html#method.set_handle
	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn lock_owner(&self) -> Option<u64> {
		self.lock_owner
	}

	/// Platform-specific flags passed to [`FuseHandlers::open`]. See
	/// [`OpenRequest::flags`] for details.
	///
	/// [`FuseHandlers::open`]: ../trait.FuseHandlers.html#method.open
	/// [`OpenRequest::flags`]: struct.OpenRequest.html#method.flags
	pub fn open_flags(&self) -> u32 {
		self.open_flags
	}
}

impl fmt::Debug for ReadRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadRequest")
			.field("node_id", &self.node_id)
			.field("size", &self.size)
			.field("offset", &self.offset)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.field("open_flags", &DebugHexU32(self.open_flags))
			.finish()
	}
}

#[repr(C)]
pub(crate) struct fuse_read_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) offset: u64,
	pub(crate) size: u32,
	pub(crate) padding: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for ReadRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_READ);

		let node_id = try_node_id(header.nodeid)?;

		// FUSE v7.9 added new fields to `fuse_read_in`.
		if dec.version().minor() < 9 {
			let raw: &'a fuse_read_in_v7p1 = dec.next_sized()?;
			return Ok(Self {
				phantom: PhantomData,
				node_id,
				size: raw.size,
				offset: raw.offset,
				handle: raw.fh,
				lock_owner: None,
				open_flags: 0,
			});
		}

		let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;

		let mut lock_owner = None;
		if raw.read_flags & fuse_kernel::FUSE_READ_LOCKOWNER != 0 {
			lock_owner = Some(raw.lock_owner);
		}

		Ok(Self {
			phantom: PhantomData,
			node_id,
			size: raw.size,
			offset: raw.offset,
			handle: raw.fh,
			lock_owner,
			open_flags: raw.flags,
		})
	}
}

// }}}

// ReadResponse {{{

/// Response type for [`FuseHandlers::read`].
///
/// [`FuseHandlers::read`]: ../trait.FuseHandlers.html#method.read
pub struct ReadResponse<'a> {
	bytes: &'a [u8],
}

impl<'a> ReadResponse<'a> {
	pub fn from_bytes(bytes: &'a [u8]) -> ReadResponse<'a> {
		Self { bytes }
	}

	// TODO; from &[std::io::IoSlice]

	// TODO: from file descriptor (for splicing)
}

impl fmt::Debug for ReadResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let bytes = DebugBytesAsString(self.bytes);
		fmt.debug_struct("ReadResponse")
			.field("bytes", &bytes)
			.finish()
	}
}

impl fuse_io::EncodeResponse for ReadResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_bytes(self.bytes)
	}
}

// }}}
