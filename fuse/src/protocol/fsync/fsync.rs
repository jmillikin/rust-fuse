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

#[cfg(rust_fuse_test = "fsync_test")]
mod fsync_test;

// FsyncRequest {{{

const FSYNC_DATASYNC: u32 = 1 << 0;

pub struct FsyncRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	handle: u64,
	flags: FsyncRequestFlags,
}

impl FsyncRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn flags(&self) -> &FsyncRequestFlags {
		&self.flags
	}
}

bitflags_struct! {
	/// Optional flags set on [`FsyncRequest`].
	///
	/// [`FsyncRequest`]: struct.FsyncRequest.html
	pub struct FsyncRequestFlags(u32);

	FSYNC_DATASYNC: datasync,
}

impl fmt::Debug for FsyncRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("flags", &self.flags)
			.finish()
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for FsyncRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_FSYNC);

		let raw: &fuse_kernel::fuse_fsync_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
			handle: raw.fh,
			flags: FsyncRequestFlags::from_bits(raw.fsync_flags),
		})
	}
}

// }}}

// FsyncResponse {{{

pub struct FsyncResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> FsyncResponse<'a> {
	pub fn new() -> FsyncResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for FsyncResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncResponse").finish()
	}
}

impl fuse_io::EncodeResponse for FsyncResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		enc.encode_header_only()
	}
}

// }}}
