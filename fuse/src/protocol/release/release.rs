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
mod release_test;

// ReleaseRequest {{{

/// Request type for [`FuseHandlers::release`].
///
/// [`FuseHandlers::release`]: ../trait.FuseHandlers.html#method.release
pub struct ReleaseRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	handle: u64,
	lock_owner: Option<u64>,
	flags: u32,
}

impl ReleaseRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: protocol/struct.OpenResponse.html#method.set_handle
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
	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl fmt::Debug for ReleaseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.field("flags", &DebugHexU32(self.flags))
			.finish()
	}
}

#[repr(C)]
pub(crate) struct fuse_release_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) flags: u32,
	pub(crate) padding: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for ReleaseRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_RELEASE);

		let node_id = try_node_id(header.nodeid)?;

		// FUSE v7.8 added new fields to `fuse_release_in`.
		if dec.version().minor() < 8 {
			let raw: &'a fuse_release_in_v7p1 = dec.next_sized()?;
			return Ok(Self {
				phantom: PhantomData,
				node_id,
				handle: raw.fh,
				lock_owner: None,
				flags: raw.flags,
			});
		}

		let raw: &'a fuse_kernel::fuse_release_in = dec.next_sized()?;

		let mut lock_owner = None;
		if raw.release_flags & fuse_kernel::FUSE_RELEASE_FLOCK_UNLOCK != 0 {
			lock_owner = Some(raw.lock_owner);
		}

		Ok(Self {
			phantom: PhantomData,
			node_id,
			handle: raw.fh,
			lock_owner,
			flags: raw.flags,
		})
	}
}

// }}}

// ReleaseResponse {{{

/// Response type for [`FuseHandlers::release`].
///
/// [`FuseHandlers::release`]: ../trait.FuseHandlers.html#method.release
pub struct ReleaseResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl ReleaseResponse<'_> {
	pub fn new() -> Self {
		ReleaseResponse {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for ReleaseResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseResponse").finish()
	}
}

impl fuse_io::EncodeResponse for ReleaseResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_header_only()
	}
}

// }}}
