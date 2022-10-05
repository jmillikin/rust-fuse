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

//! Implements the `FUSE_GETLK` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::file_lock::{Lock, F_RDLCK, F_UNLCK, F_WRLCK};

// GetlkRequest {{{

/// Request type for `FUSE_GETLK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETLK` operation.
pub struct GetlkRequest<'a> {
	raw: &'a fuse_kernel::fuse_lk_in,
	node_id: node::Id,
	lock: Lock,
}

impl GetlkRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		self.node_id
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	#[must_use]
	pub fn owner(&self) -> u64 {
		self.raw.owner
	}

	#[must_use]
	pub fn lock(&self) -> &Lock {
		&self.lock
	}
}

request_try_from! { GetlkRequest : fuse }

impl decode::Sealed for GetlkRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for GetlkRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_GETLK)?;

		let raw: &fuse_kernel::fuse_lk_in = dec.next_sized()?;
		let node_id = decode::node_id(dec.header().nodeid)?;

		let lock = match Lock::parse(raw.lk) {
			Some(l) => l,
			_ => return Err(server::RequestError::InvalidLockType),
		};
		Ok(Self { raw, node_id, lock })
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

// }}}

// GetlkResponse {{{

/// Response type for `FUSE_GETLK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETLK` operation.
pub struct GetlkResponse<'a> {
	phantom: PhantomData<&'a ()>,
	lock: Option<Lock>,
}

impl<'a> GetlkResponse<'a> {
	#[must_use]
	pub fn new() -> GetlkResponse<'a> {
		Self {
			phantom: PhantomData,
			lock: None,
		}
	}

	#[must_use]
	pub fn lock(&self) -> &Option<Lock> {
		&self.lock
	}

	pub fn set_lock(&mut self, lock: Option<Lock>) {
		self.lock = lock;
	}
}

response_send_funcs!(GetlkResponse<'_>);

impl fmt::Debug for GetlkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkResponse")
			.field("lock", &self.lock)
			.finish()
	}
}

impl GetlkResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
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
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&fuse_kernel::fuse_lk_out { lk: lock })
	}
}
// }}}
