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

//! Implements the `FUSE_SYMLINK` operation.

use core::fmt;

use crate::internal::debug;
use crate::kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// SymlinkRequest {{{

/// Request type for `FUSE_SYMLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYMLINK` operation.
pub struct SymlinkRequest<'a> {
	parent_id: crate::NodeId,
	name: &'a crate::NodeName,
	content: &'a [u8],
}

impl SymlinkRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> crate::NodeId {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
		self.name
	}

	#[must_use]
	pub fn content(&self) -> &[u8] {
		self.content
	}
}

impl server::sealed::Sealed for SymlinkRequest<'_> {}

impl<'a> server::FuseRequest<'a> for SymlinkRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_SYMLINK)?;
		let content = dec.next_nul_terminated_bytes()?.to_bytes_without_nul();
		let name = dec.next_node_name()?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name,
			content,
		})
	}
}

impl fmt::Debug for SymlinkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkRequest")
			.field("parent_id", &self.parent_id)
			.field("name", &self.name)
			.field("content", &debug::bytes(self.content))
			.finish()
	}
}

// }}}

// SymlinkResponse {{{

/// Response type for `FUSE_SYMLINK`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYMLINK` operation.
pub struct SymlinkResponse {
	entry: crate::Entry,
}

impl SymlinkResponse {
	#[must_use]
	pub fn new(entry: crate::Entry) -> SymlinkResponse {
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

impl fmt::Debug for SymlinkResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SymlinkResponse")
			.field("entry", &self.entry())
			.finish()
	}
}

impl server::sealed::Sealed for SymlinkResponse {}

impl server::FuseResponse for SymlinkResponse {
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
