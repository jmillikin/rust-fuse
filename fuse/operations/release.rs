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

//! Implements the `FUSE_RELEASE` operation.

use core::fmt;

use crate::internal::compat;
use crate::internal::debug;
use crate::kernel;
use crate::lock;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// ReleaseRequest {{{

/// Request type for `FUSE_RELEASE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASE` operation.
pub struct ReleaseRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_release_in<'a>>,
}

impl ReleaseRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		crate::NodeId::new(self.header.nodeid).unwrap_or(crate::NodeId::ROOT)
	}

	/// The value passed to [`OpenResponse::set_handle`], or zero if not set.
	///
	/// [`OpenResponse::set_handle`]: crate::operations::open::OpenResponse::set_handle
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.as_v7p1().fh
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<lock::Owner> {
		let body = self.body.as_v7p8()?;
		if body.release_flags & kernel::FUSE_RELEASE_FLOCK_UNLOCK == 0 {
			return None;
		}
		Some(lock::Owner::new(body.lock_owner))
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		self.body.as_v7p1().flags
	}
}

impl server::sealed::Sealed for ReleaseRequest<'_> {}

impl<'a> server::CuseRequest<'a> for ReleaseRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		options: server::CuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request, options.version_minor(), true)
	}
}

impl<'a> server::FuseRequest<'a> for ReleaseRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode_request(request, options.version_minor(), false)
	}
}

impl<'a> ReleaseRequest<'a> {
	fn decode_request(
		request: server::Request<'a>,
		version_minor: u32,
		is_cuse: bool,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_RELEASE)?;

		let header = dec.header();
		if !is_cuse {
			decode::node_id(header.nodeid)?;
		}

		let body = if version_minor >= 8 {
			let body_v7p8 = dec.next_sized()?;
			compat::Versioned::new_release_v7p8(version_minor, body_v7p8)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_release_v7p1(version_minor, body_v7p1)
		};

		Ok(Self { header, body })
	}
}

impl fmt::Debug for ReleaseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.handle())
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}

// ReleaseResponse {{{

/// Response type for `FUSE_RELEASE`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RELEASE` operation.
pub struct ReleaseResponse {
	_priv: (),
}

impl ReleaseResponse {
	#[must_use]
	pub fn new() -> ReleaseResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for ReleaseResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReleaseResponse").finish()
	}
}

impl server::sealed::Sealed for ReleaseResponse {}

impl server::CuseResponse for ReleaseResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::CuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

impl server::FuseResponse for ReleaseResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
