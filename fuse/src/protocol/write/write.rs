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

#[cfg(rust_fuse_test = "write_test")]
mod write_test;

// WriteRequest {{{

/// Request type for [`FuseHandlers::write`].
///
/// [`FuseHandlers::write`]: ../../trait.FuseHandlers.html#method.write
pub struct WriteRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	offset: u64,
	handle: u64,
	value: &'a [u8],
	flags: WriteRequestFlags,
	lock_owner: Option<u64>,
	open_flags: u32,
}

impl WriteRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
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

	pub fn value(&self) -> &[u8] {
		self.value
	}

	pub fn flags(&self) -> &WriteRequestFlags {
		&self.flags
	}

	pub fn lock_owner(&self) -> Option<u64> {
		self.lock_owner
	}

	/// Platform-specific flags passed to [`FuseHandlers::open`]. See
	/// [`OpenRequest::flags`] for details.
	///
	/// [`FuseHandlers::open`]: ../../trait.FuseHandlers.html#method.open
	/// [`OpenRequest::flags`]: struct.OpenRequest.html#method.flags
	pub fn open_flags(&self) -> u32 {
		self.open_flags
	}
}

bitflags_struct! {
	/// Optional flags set on [`WriteRequest`].
	///
	/// [`WriteRequest`]: struct.OpendirResponse.html
	pub struct WriteRequestFlags(u32);

	fuse_kernel::FUSE_WRITE_CACHE: write_cache,
}

impl fmt::Debug for WriteRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteRequest")
			.field("node_id", &self.node_id)
			.field("offset", &self.offset)
			.field("handle", &self.handle)
			.field("value", &self.value)
			.field("flags", &self.flags)
			.field("lock_owner", &self.lock_owner())
			.field("open_flags", &DebugHexU32(self.open_flags))
			.finish()
	}
}

#[repr(C)]
struct fuse_write_in_v7p1 {
	fh: u64,
	offset: u64,
	size: u32,
	write_flags: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for WriteRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_WRITE);

		let node_id = if dec.is_cuse() {
			crate::ROOT_ID
		} else {
			try_node_id(header.nodeid)?
		};

		if dec.version().minor() < 9 {
			let raw: &'a fuse_write_in_v7p1 = dec.next_sized()?;
			let value = dec.next_bytes(raw.size)?;
			return Ok(Self {
				phantom: PhantomData,
				node_id,
				offset: raw.offset,
				handle: raw.fh,
				value,
				flags: WriteRequestFlags::from_bits(raw.write_flags),
				lock_owner: None,
				open_flags: 0,
			});
		}

		let raw: &'a fuse_kernel::fuse_write_in = dec.next_sized()?;
		let value = dec.next_bytes(raw.size)?;

		let mut lock_owner = None;
		if raw.write_flags & fuse_kernel::FUSE_WRITE_LOCKOWNER != 0 {
			lock_owner = Some(raw.lock_owner)
		}

		Ok(Self {
			phantom: PhantomData,
			node_id,
			offset: raw.offset,
			handle: raw.fh,
			value,
			flags: WriteRequestFlags::from_bits(raw.write_flags),
			lock_owner,
			open_flags: raw.flags,
		})
	}
}

// }}}

// WriteResponse {{{

/// Response type for [`FuseHandlers::write`].
///
/// [`FuseHandlers::write`]: ../../trait.FuseHandlers.html#method.write
pub struct WriteResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_write_out,
}

impl<'a> WriteResponse<'a> {
	pub fn new() -> WriteResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_write_out {
				size: 0,
				padding: 0,
			},
		}
	}

	pub fn set_size(&mut self, size: u32) {
		self.raw.size = size;
	}
}

impl fmt::Debug for WriteResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("WriteResponse")
			.field("size", &self.raw.size)
			.finish()
	}
}

impl fuse_io::EncodeResponse for WriteResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_sized(&self.raw)
	}
}

// }}}
