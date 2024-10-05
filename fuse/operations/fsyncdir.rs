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

//! Implements the `FUSE_FSYNCDIR` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// FsyncdirRequest {{{

/// Request type for `FUSE_FSYNCDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FSYNCDIR` operation.
pub struct FsyncdirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_fsync_in,
}

impl FsyncdirRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub fn flags(&self) -> FsyncdirRequestFlags {
		FsyncdirRequestFlags {
			bits: self.body.fsync_flags,
		}
	}
}

impl server::sealed::Sealed for FsyncdirRequest<'_> {}

impl<'a> server::FuseRequest<'a> for FsyncdirRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_FSYNCDIR)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for FsyncdirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// FsyncdirResponse {{{

/// Response type for `FUSE_FSYNCDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_FSYNCDIR` operation.
pub struct FsyncdirResponse {
	_priv: (),
}

impl FsyncdirResponse {
	#[must_use]
	pub fn new() -> FsyncdirResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for FsyncdirResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FsyncdirResponse").finish()
	}
}

impl server::sealed::Sealed for FsyncdirResponse {}

impl server::FuseResponse for FsyncdirResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}

// FsyncdirRequestFlags {{{

/// Optional flags set on [`FsyncdirRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncdirRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FsyncdirRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(FsyncdirRequestFlag, FsyncdirRequestFlags, u32, {
		FDATASYNC = fuse_kernel::FUSE_FSYNC_FDATASYNC;
	});
}

// }}}
