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

use core::cmp;
use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::lock;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// GetlkRequest {{{

/// Request type for `FUSE_GETLK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETLK` operation.
pub struct GetlkRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_lk_in,
	lock_mode: lock::Mode,
	lock_range: lock::Range,
}

impl GetlkRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[inline]
	#[must_use]
	pub fn owner(&self) -> lock::Owner {
		lock::Owner::new(self.body.owner)
	}

	#[inline]
	#[must_use]
	pub fn lock_mode(&self) -> lock::Mode {
		self.lock_mode
	}

	#[inline]
	#[must_use]
	pub fn lock_range(&self) -> lock::Range {
		self.lock_range
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

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body: &fuse_kernel::fuse_lk_in = dec.next_sized()?;
		let lock_mode = lock::decode_mode(&body.lk)?;
		let lock_range = lock::decode_range(&body.lk)?;
		Ok(Self { header, body, lock_mode, lock_range })
	}
}

impl fmt::Debug for GetlkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("owner", &self.owner())
			.field("lock_mode", &self.lock_mode())
			.field("lock_range", &self.lock_range())
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
	lock: Option<lock::Lock>,
}

impl<'a> GetlkResponse<'a> {
	#[inline]
	#[must_use]
	pub fn new(lock: Option<lock::Lock>) -> GetlkResponse<'a> {
		Self {
			phantom: PhantomData,
			lock,
		}
	}

	#[inline]
	#[must_use]
	pub fn lock(&self) -> Option<lock::Lock> {
		self.lock
	}
}

response_send_funcs!(GetlkResponse<'_>);

impl fmt::Debug for GetlkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkResponse")
			.field("lock", &self.lock())
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
				r#type: lock::F_UNLCK,
				start: 0,
				end: 0,
				pid: 0,
			},
			Some(lock) => {
				fuse_kernel::fuse_file_lock {
					r#type: match lock.mode() {
						lock::Mode::Exclusive => lock::F_WRLCK,
						lock::Mode::Shared => lock::F_RDLCK,
					},
					start: cmp::min(
						lock.range().start(),
						lock::OFFSET_MAX,
					),
					end: cmp::min(
						lock.range().end().unwrap_or(lock::OFFSET_MAX),
						lock::OFFSET_MAX,
					),
					pid: lock.process_id().map(|x| x.get()).unwrap_or(0),
				}
			},
		};
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&fuse_kernel::fuse_lk_out { lk: lock })
	}
}

// }}}
