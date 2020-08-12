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

use crate::protocol::node;
use crate::protocol::prelude::*;
use crate::protocol::release::fuse_release_in_v7p1;

#[cfg(test)]
mod releasedir_test;

// ReleasedirRequest {{{

/// Request type for [`FuseHandlers::releasedir`].
///
/// [`FuseHandlers::releasedir`]: ../trait.FuseHandlers.html#method.releasedir
pub struct ReleasedirRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: node::NodeId,
	handle: u64,
	lock_owner: Option<u64>,
	flags: u32,
}

impl ReleasedirRequest<'_> {
	pub fn node_id(&self) -> node::NodeId {
		self.node_id
	}

	/// The value passed to [`OpendirResponse::set_handle`], or zero if not set.
	///
	/// [`OpendirResponse::set_handle`]: protocol/struct.OpendirResponse.html#method.set_handle
	pub fn handle(&self) -> u64 {
		self.handle
	}

	pub fn lock_owner(&self) -> Option<u64> {
		self.lock_owner
	}

	/// Platform-specific flags passed to [`FuseHandlers::opendir`]. See
	/// [`OpendirRequest::flags`] for details.
	///
	/// [`FuseHandlers::opendir`]: ../trait.FuseHandlers.html#method.opendir
	/// [`OpendirRequest::flags`]: struct.OpendirRequest.html#method.flags
	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl fmt::Debug for ReleasedirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleasedirRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.field("flags", &DebugHexU32(self.flags))
			.finish()
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ReleasedirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_RELEASEDIR);

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

// ReleasedirResponse {{{

/// Response type for [`FuseHandlers::releasedir`].
///
/// [`FuseHandlers::releasedir`]: ../trait.FuseHandlers.html#method.releasedir
pub struct ReleasedirResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl ReleasedirResponse<'_> {
	pub fn new() -> Self {
		ReleasedirResponse {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for ReleasedirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleasedirResponse").finish()
	}
}

impl fuse_io::EncodeResponse for ReleasedirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_header_only()
	}
}

// }}}
