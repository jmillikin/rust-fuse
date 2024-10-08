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

//! Implements the `FUSE_SETLK` and `FUSE_SETLKW` operations.

use core::fmt;

use crate::kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// SetlkRequest {{{

/// Request type for `FUSE_SETLK` and `FUSE_SETLKW`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETLK` and `FUSE_SETLKW` operations.
pub struct SetlkRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_lk_in,
	lock: crate::Lock,
}

impl SetlkRequest<'_> {
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
	pub fn may_block(&self) -> bool {
		self.header.opcode == kernel::fuse_opcode::FUSE_SETLKW
	}

	#[inline]
	#[must_use]
	pub fn owner(&self) -> crate::LockOwner {
		crate::LockOwner(self.body.owner)
	}

	#[inline]
	#[must_use]
	pub fn lock(&self) -> crate::Lock {
		self.lock
	}

	#[inline]
	#[must_use]
	pub fn flags(&self) -> SetlkRequestFlags {
		SetlkRequestFlags {
			bits: self.body.lk_flags,
		}
	}
}

impl server::sealed::Sealed for SetlkRequest<'_> {}

impl<'a> server::FuseRequest<'a> for SetlkRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();

		let header = dec.header();
		if header.opcode != kernel::fuse_opcode::FUSE_SETLKW {
			dec.expect_opcode(kernel::fuse_opcode::FUSE_SETLK)?;
		}

		decode::node_id(header.nodeid)?;

		let body: &kernel::fuse_lk_in = dec.next_sized()?;
		let lock = crate::Lock::decode(&body.lk)?;
		Ok(Self { header, body, lock })
	}
}

/// Optional flags set on [`SetlkRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SetlkRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SetlkRequestFlag {
	mask: u32,
}

mod flags {
	use crate::kernel;
	bitflags!(SetlkRequestFlag, SetlkRequestFlags, u32, {
		LK_FLOCK = kernel::FUSE_LK_FLOCK;
	});
}

impl fmt::Debug for SetlkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetlkRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("may_block", &self.may_block())
			.field("owner", &self.owner())
			.field("lock", &self.lock())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// SetlkResponse {{{

/// Response type for `FUSE_SETLK` and `FUSE_SETLKW`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETLK` and `FUSE_SETLKW` operations.
pub struct SetlkResponse {
	_priv: (),
}

impl SetlkResponse {
	#[must_use]
	pub fn new() -> SetlkResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for SetlkResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetlkResponse").finish()
	}
}

impl server::sealed::Sealed for SetlkResponse {}

impl server::FuseResponse for SetlkResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
