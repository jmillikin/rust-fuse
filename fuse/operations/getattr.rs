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

//! Implements the `FUSE_GETATTR` operation.

use core::fmt;
use core::time;

use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// GetattrRequest {{{

/// Request type for `FUSE_GETATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETATTR` operation.
pub struct GetattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_getattr_in<'a>>,
}

impl GetattrRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> Option<u64> {
		let body = self.body.as_v7p9()?;
		if (body.getattr_flags & fuse_kernel::FUSE_GETATTR_FH) > 0 {
			return Some(body.fh);
		}
		None
	}
}

impl server::sealed::Sealed for GetattrRequest<'_> {}

impl<'a> server::FuseRequest<'a> for GetattrRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let version_minor = options.version_minor();
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_GETATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = if version_minor >= 9 {
			let body_v7p9 = dec.next_sized()?;
			compat::Versioned::new_getattr_v7p9(version_minor, body_v7p9)
		} else {
			compat::Versioned::new_getattr_v7p1(version_minor)
		};

		Ok(Self { header, body })
	}
}

impl fmt::Debug for GetattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetattrRequest")
			.field("node_id", &self.node_id())
			.field("handle", &format_args!("{:?}", &self.handle()))
			.finish()
	}
}

// }}}

// GetattrResponse {{{

/// Response type for `FUSE_GETATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETATTR` operation.
pub struct GetattrResponse {
	attr_out: node::FuseAttrOut,
}

impl GetattrResponse {
	#[inline]
	#[must_use]
	pub fn new(attributes: node::Attributes) -> GetattrResponse {
		Self {
			attr_out: node::FuseAttrOut::new(attributes),
		}
	}

	#[inline]
	#[must_use]
	pub fn attributes(&self) -> &node::Attributes {
		self.attr_out.attributes()
	}

	#[inline]
	#[must_use]
	pub fn attributes_mut(&mut self) -> &mut node::Attributes {
		self.attr_out.attributes_mut()
	}

	#[inline]
	#[must_use]
	pub fn cache_timeout(&self) -> time::Duration {
		self.attr_out.cache_timeout()
	}

	#[inline]
	pub fn set_cache_timeout(&mut self, timeout: time::Duration) {
		self.attr_out.set_cache_timeout(timeout)
	}
}

impl fmt::Debug for GetattrResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetattrResponse")
			.field("attributes", self.attributes())
			.field("cache_timeout", &self.cache_timeout())
			.finish()
	}
}

impl server::sealed::Sealed for GetattrResponse {}

impl server::FuseResponse for GetattrResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		if options.version_minor() >= 9 {
			return encode::sized(header, self.attr_out.as_v7p9());
		}
		encode::sized(header, self.attr_out.as_v7p1())
	}
}

// }}}
