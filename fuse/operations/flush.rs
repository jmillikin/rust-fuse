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

use core::fmt;
use core::marker::PhantomData;

use crate::kernel;
use crate::server;
use crate::server::decode;

// FlushRequest {{{

/// Request type for `FUSE_FLUSH`.
#[derive(Clone, Copy)]
pub struct FlushRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: crate::NodeId,
	handle: u64,
	lock_owner: crate::LockOwner,
}

impl FlushRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.handle
	}

	#[must_use]
	pub fn lock_owner(&self) -> crate::LockOwner {
		self.lock_owner
	}
}

try_from_cuse_request!(FlushRequest<'a>, |request| {
	Self::try_from(request.inner, true)
});

try_from_fuse_request!(FlushRequest<'a>, |request| {
	Self::try_from(request.inner, false)
});

impl<'a> FlushRequest<'a> {
	fn try_from(
		request: server::Request<'a>,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_FLUSH)?;

		let node_id = if is_cuse {
			crate::NodeId::ROOT
		} else {
			decode::node_id(dec.header().nodeid)?
		};

		let raw: &kernel::fuse_flush_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id,
			handle: raw.fh,
			lock_owner: crate::LockOwner(raw.lock_owner),
		})
	}
}

impl fmt::Debug for FlushRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FlushRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.handle)
			.field("lock_owner", &self.lock_owner)
			.finish()
	}
}

// }}}
