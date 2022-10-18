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

//! Implements the `FUSE_REMOVEXATTR` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;
use crate::xattr;

// RemovexattrRequest {{{

/// Request type for `FUSE_REMOVEXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_REMOVEXATTR` operation.
pub struct RemovexattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	name: &'a xattr::Name,
}

impl RemovexattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &xattr::Name {
		self.name
	}
}

impl server::sealed::Sealed for RemovexattrRequest<'_> {}

impl<'a> server::FuseRequest<'a> for RemovexattrRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_REMOVEXATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let name_bytes = dec.next_nul_terminated_bytes()?;
		let name = xattr::Name::from_bytes(name_bytes.to_bytes_without_nul())?;
		Ok(Self { header, name })
	}
}

impl fmt::Debug for RemovexattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.finish()
	}
}

// }}}

// RemovexattrResponse {{{

/// Response type for `FUSE_REMOVEXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_REMOVEXATTR` operation.
pub struct RemovexattrResponse {
	_priv: (),
}

impl RemovexattrResponse {
	#[inline]
	#[must_use]
	pub fn new() -> RemovexattrResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for RemovexattrResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RemovexattrResponse").finish()
	}
}

impl server::sealed::Sealed for RemovexattrResponse {}

impl server::FuseResponse for RemovexattrResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
