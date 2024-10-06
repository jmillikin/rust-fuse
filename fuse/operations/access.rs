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

//! Implements the `FUSE_ACCESS` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// AccessRequest {{{

/// Request type for `FUSE_ACCESS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_ACCESS` operation.
pub struct AccessRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: crate::NodeId,
	mask: u32,
}

impl AccessRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}

	#[must_use]
	pub fn mask(&self) -> u32 {
		self.mask
	}
}

impl server::sealed::Sealed for AccessRequest<'_> {}

impl<'a> server::FuseRequest<'a> for AccessRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_ACCESS)?;
		let raw: &'a kernel::fuse_access_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: decode::node_id(dec.header().nodeid)?,
			mask: raw.mask,
		})
	}
}

impl fmt::Debug for AccessRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("AccessRequest")
			.field("node_id", &self.node_id)
			.field("mask", &self.mask)
			.finish()
	}
}

// }}}

// AccessResponse {{{

/// Response type for `FUSE_ACCESS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_ACCESS` operation.
pub struct AccessResponse {
	_priv: (),
}

impl AccessResponse {
	#[must_use]
	pub fn new() -> AccessResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for AccessResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("AccessResponse").finish()
	}
}

impl server::sealed::Sealed for AccessResponse {}

impl server::FuseResponse for AccessResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
