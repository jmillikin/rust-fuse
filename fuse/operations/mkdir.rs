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

//! Implements the `FUSE_MKDIR` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// MkdirRequest {{{

/// Request type for `FUSE_MKDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_MKDIR` operation.
pub struct MkdirRequest<'a> {
	parent_id: node::Id,
	name: &'a node::Name,
	raw: fuse_kernel::fuse_mkdir_in,
}

impl MkdirRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}

	#[must_use]
	pub fn mode(&self) -> node::Mode {
		node::Mode::new(self.raw.mode)
	}

	#[must_use]
	pub fn umask(&self) -> u32 {
		self.raw.umask
	}
}

impl server::sealed::Sealed for MkdirRequest<'_> {}

impl<'a> server::FuseRequest<'a> for MkdirRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_MKDIR)?;

		let raw: &fuse_kernel::fuse_mkdir_in = dec.next_sized()?;
		let name = dec.next_node_name()?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name,
			raw: *raw,
		})
	}
}

impl fmt::Debug for MkdirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MkdirRequest")
			.field("parent_id", &self.parent_id())
			.field("name", &self.name())
			.field("mode", &self.mode())
			.field("umask", &format_args!("{:#o}", &self.raw.umask))
			.finish()
	}
}

// }}}

// MkdirResponse {{{

/// Response type for `FUSE_MKDIR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_MKDIR` operation.
pub struct MkdirResponse {
	entry: node::Entry,
}

impl MkdirResponse {
	#[inline]
	#[must_use]
	pub fn new(entry: node::Entry) -> MkdirResponse {
		Self { entry }
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> &node::Entry {
		&self.entry
	}

	#[inline]
	#[must_use]
	pub fn entry_mut(&mut self) -> &mut node::Entry {
		&mut self.entry
	}
}

impl fmt::Debug for MkdirResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MkdirResponse")
			.field("entry", &self.entry())
			.finish()
	}
}

impl server::sealed::Sealed for MkdirResponse {}

impl server::FuseResponse for MkdirResponse {
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
