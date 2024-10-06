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

//! Implements the `FUSE_UNLINK` operation.

use core::fmt;

use crate::kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// UnlinkRequest {{{

/// Request type for `FUSE_UNLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_UNLINK` operation.
#[derive(Debug)]
pub struct UnlinkRequest<'a> {
	parent_id: crate::NodeId,
	name: &'a crate::NodeName,
}

impl UnlinkRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> crate::NodeId {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
		self.name
	}
}

impl server::sealed::Sealed for UnlinkRequest<'_> {}

impl<'a> server::FuseRequest<'a> for UnlinkRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_UNLINK)?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name: dec.next_node_name()?,
		})
	}
}

// }}}

// UnlinkResponse {{{

/// Response type for `FUSE_UNLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_UNLINK` operation.
pub struct UnlinkResponse {
	_priv: (),
}

impl UnlinkResponse {
	#[must_use]
	pub fn new() -> UnlinkResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for UnlinkResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("UnlinkResponse").finish()
	}
}

impl server::sealed::Sealed for UnlinkResponse {}

impl server::FuseResponse for UnlinkResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
