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

use crate::protocol::common::file_lock::{Lock, F_RDLCK, F_UNLCK, F_WRLCK};
use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "getlk_test")]
mod getlk_test;

// GetlkRequest {{{

/// Request type for [`FuseHandlers::getlk`].
///
/// [`FuseHandlers::getlk`]: ../../trait.FuseHandlers.html#method.getlk
pub struct GetlkRequest<'a> {
	raw: &'a fuse_kernel::fuse_lk_in,
	node_id: NodeId,
	lock: Lock,
}

impl GetlkRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn owner(&self) -> u64 {
		self.raw.owner
	}

	pub fn lock(&self) -> &Lock {
		&self.lock
	}
}

impl fmt::Debug for GetlkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("owner", &self.raw.owner)
			.field("lock", &self.lock)
			.finish()
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for GetlkRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_GETLK);
		let raw: &fuse_kernel::fuse_lk_in = dec.next_sized()?;
		let node_id = try_node_id(header.nodeid)?;

		let lock_type = raw.lk.r#type;
		let lock = match Lock::parse(raw.lk) {
			Some(l) => l,
			_ => return Err(Error::invalid_lock_type(lock_type)),
		};
		Ok(Self { raw, node_id, lock })
	}
}

// }}}

// GetlkResponse {{{

/// Response type for [`FuseHandlers::getlk`].
///
/// [`FuseHandlers::getlk`]: ../../trait.FuseHandlers.html#method.getlk
pub struct GetlkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	lock: Option<Lock>,
}

impl<'a> GetlkResponse<'a> {
	pub fn new() -> GetlkResponse<'a> {
		Self {
			phantom: PhantomData,
			lock: None,
		}
	}

	pub fn lock(&self) -> &Option<Lock> {
		&self.lock
	}

	pub fn set_lock(&mut self, lock: Option<Lock>) {
		self.lock = lock;
	}
}

impl fmt::Debug for GetlkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkResponse")
			.field("lock", &self.lock)
			.finish()
	}
}

impl fuse_io::EncodeResponse for GetlkResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		let lock = match self.lock {
			None => fuse_kernel::fuse_file_lock {
				start: 0,
				end: 0,
				r#type: F_UNLCK,
				pid: 0,
			},
			Some(Lock::Shared { range, process_id }) => {
				fuse_kernel::fuse_file_lock {
					start: range.start,
					end: range.end,
					r#type: F_RDLCK,
					pid: process_id,
				}
			},
			Some(Lock::Exclusive { range, process_id }) => {
				fuse_kernel::fuse_file_lock {
					start: range.start,
					end: range.end,
					r#type: F_WRLCK,
					pid: process_id,
				}
			},
		};
		enc.encode_sized(&fuse_kernel::fuse_lk_out { lk: lock })
	}
}

// }}}
