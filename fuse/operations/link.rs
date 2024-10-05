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

//! Implements the `FUSE_LINK` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// LinkRequest {{{

/// Request type for `FUSE_LINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LINK` operation.
#[derive(Debug)]
pub struct LinkRequest<'a> {
	node_id: crate::NodeId,
	new_parent_id: crate::NodeId,
	new_name: &'a crate::NodeName,
}

impl LinkRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		self.node_id
	}

	#[must_use]
	pub fn new_parent_id(&self) -> crate::NodeId {
		self.new_parent_id
	}

	#[must_use]
	pub fn new_name(&self) -> &crate::NodeName {
		self.new_name
	}
}

impl server::sealed::Sealed for LinkRequest<'_> {}

impl<'a> server::FuseRequest<'a> for LinkRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LINK)?;

		let raw: &fuse_kernel::fuse_link_in = dec.next_sized()?;
		let name = dec.next_node_name()?;
		Ok(Self {
			node_id: decode::node_id(raw.oldnodeid)?,
			new_parent_id: decode::node_id(dec.header().nodeid)?,
			new_name: name,
		})
	}
}

// }}}

// LinkResponse {{{

/// Response type for `FUSE_LINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LINK` operation.
pub struct LinkResponse {
	entry: crate::Entry,
}

impl LinkResponse {
	#[inline]
	#[must_use]
	pub fn new(entry: crate::Entry) -> LinkResponse {
		Self { entry }
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> &crate::Entry {
		&self.entry
	}

	#[inline]
	#[must_use]
	pub fn entry_mut(&mut self) -> &mut crate::Entry {
		&mut self.entry
	}
}

impl fmt::Debug for LinkResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LinkResponse")
			.field("entry", &self.entry())
			.finish()
	}
}

impl server::sealed::Sealed for LinkResponse {}

impl server::FuseResponse for LinkResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		if options.version_minor() >= 9 {
			return encode::sized(header, self.entry.as_v7p9());
		}
		encode::sized(header, self.entry.as_v7p1())
	}
}

// }}}
