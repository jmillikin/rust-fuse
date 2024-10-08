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

use crate::kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// GetlkRequest {{{

/// Request type for `FUSE_GETLK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETLK` operation.
pub struct GetlkRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_lk_in,
	lock_range: crate::LockRange,
}

impl GetlkRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[inline]
	#[must_use]
	pub fn owner(&self) -> crate::LockOwner {
		crate::LockOwner(self.body.owner)
	}

	#[inline]
	#[must_use]
	pub fn lock_mode(&self) -> crate::LockMode {
		crate::LockMode(self.body.lk.r#type)
	}

	#[inline]
	#[must_use]
	pub fn lock_range(&self) -> crate::LockRange {
		self.lock_range
	}
}

impl server::sealed::Sealed for GetlkRequest<'_> {}

impl<'a> server::FuseRequest<'a> for GetlkRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_GETLK)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body: &kernel::fuse_lk_in = dec.next_sized()?;
		let lock_range = crate::LockRange::decode(&body.lk)?;
		Ok(Self { header, body, lock_range })
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
pub struct GetlkResponse {
	lock: crate::Lock,
	raw: kernel::fuse_lk_out,
}

impl GetlkResponse {
	#[inline]
	#[must_use]
	pub fn new(lock: crate::Lock) -> GetlkResponse {
		let fuse_lock = kernel::fuse_file_lock {
			r#type: lock.mode().0,
			start: cmp::min(
				lock.range().start(),
				crate::lock::OFFSET_MAX,
			),
			end: cmp::min(
				lock.range().end().unwrap_or(crate::lock::OFFSET_MAX),
				crate::lock::OFFSET_MAX,
			),
			pid: lock.process_id().map(|x| x.get()).unwrap_or(0),
		};

		Self {
			lock,
			raw: kernel::fuse_lk_out { lk: fuse_lock },
		}
	}

	#[inline]
	#[must_use]
	pub fn lock(&self) -> crate::Lock {
		self.lock
	}
}

impl fmt::Debug for GetlkResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetlkResponse")
			.field("lock", &self.lock())
			.finish()
	}
}

impl server::sealed::Sealed for GetlkResponse {}

impl server::FuseResponse for GetlkResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::sized(header, &self.raw)
	}
}

// }}}
