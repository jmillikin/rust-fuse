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

//! Implements the `FUSE_READLINK` operation.

// use core::ffi::CStr;
#[cfg(feature = "std")]
use std::ffi::CStr;

use core::fmt;
use core::marker::PhantomData;

use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// ReadlinkRequest {{{

/// Request type for `FUSE_READLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READLINK` operation.
pub struct ReadlinkRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: node::Id,
}

impl ReadlinkRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		self.node_id
	}
}

impl server::sealed::Sealed for ReadlinkRequest<'_> {}

impl<'a> server::FuseRequest<'a> for ReadlinkRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_READLINK)?;
		Ok(Self {
			phantom: PhantomData,
			node_id: decode::node_id(dec.header().nodeid)?,
		})
	}
}

impl fmt::Debug for ReadlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkRequest")
			.field("node_id", &self.node_id)
			.finish()
	}
}

// }}}

// ReadlinkResponse {{{

/// Response type for `FUSE_READLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READLINK` operation.
pub struct ReadlinkResponse<'a> {
	target: &'a [u8],
}

impl<'a> ReadlinkResponse<'a> {
	#[cfg(feature = "std")]
	#[must_use]
	pub fn new(target: &'a CStr) -> ReadlinkResponse<'a> {
		Self { target: target.to_bytes() }
	}

	#[must_use]
	pub fn from_name(target: &'a node::Name) -> ReadlinkResponse<'a> {
		Self { target: target.as_bytes() }
	}
}

impl fmt::Debug for ReadlinkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReadlinkResponse")
			.field("target", &debug::bytes(self.target))
			.finish()
	}
}

impl server::sealed::Sealed for ReadlinkResponse<'_> {}

impl server::FuseResponse for ReadlinkResponse<'_> {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::bytes(header, self.target)
	}
}

// }}}
