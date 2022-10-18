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

//! Implements the `FUSE_RMDIR` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// RmdirRequest {{{

/// Request type for `FUSE_RMDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RMDIR` operation.
#[derive(Debug)]
pub struct RmdirRequest<'a> {
	parent_id: node::Id,
	name: &'a node::Name,
}

impl RmdirRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}
}

impl server::sealed::Sealed for RmdirRequest<'_> {}

impl<'a> server::FuseRequest<'a> for RmdirRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_RMDIR)?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name: dec.next_node_name()?,
		})
	}
}

// }}}

// RmdirResponse {{{

/// Response type for `FUSE_RMDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RMDIR` operation.
pub struct RmdirResponse {
	_priv: (),
}

impl RmdirResponse {
	#[must_use]
	pub fn new() -> RmdirResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for RmdirResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RmdirResponse").finish()
	}
}

impl server::sealed::Sealed for RmdirResponse {}

impl server::FuseResponse for RmdirResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
