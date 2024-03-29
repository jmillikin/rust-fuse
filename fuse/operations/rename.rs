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

//! Implements the `FUSE_RENAME` and `FUSE_RENAME2` operations.

use core::fmt;

use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// RenameRequest {{{

/// Request type for `FUSE_RENAME` and `FUSE_RENAME2`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RENAME` and `FUSE_RENAME2` operations.
pub struct RenameRequest<'a> {
	old_directory_id: node::Id,
	old_name: &'a node::Name,
	new_directory_id: node::Id,
	new_name: &'a node::Name,
	rename_flags: u32,
}

impl RenameRequest<'_> {
	#[must_use]
	pub fn old_directory_id(&self) -> node::Id {
		self.old_directory_id
	}

	#[must_use]
	pub fn old_name(&self) -> &node::Name {
		self.old_name
	}

	#[must_use]
	pub fn new_directory_id(&self) -> node::Id {
		self.new_directory_id
	}

	#[must_use]
	pub fn new_name(&self) -> &node::Name {
		self.new_name
	}

	#[must_use]
	pub fn rename_flags(&self) -> crate::RenameFlags {
		self.rename_flags
	}
}

impl server::sealed::Sealed for RenameRequest<'_> {}

impl<'a> server::FuseRequest<'a> for RenameRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		let header = dec.header();

		let mut rename_flags = 0;
		let new_dir: u64;
		if header.opcode == fuse_kernel::FUSE_RENAME2 {
			let parsed: &fuse_kernel::fuse_rename2_in = dec.next_sized()?;
			rename_flags = parsed.flags;
			new_dir = parsed.newdir;
		} else {
			dec.expect_opcode(fuse_kernel::FUSE_RENAME)?;
			let parsed: &fuse_kernel::fuse_rename_in = dec.next_sized()?;
			new_dir = parsed.newdir;
		}
		let old_name = dec.next_node_name()?;
		let new_name = dec.next_node_name()?;
		Ok(Self {
			old_directory_id: decode::node_id(header.nodeid)?,
			old_name,
			new_directory_id: decode::node_id(new_dir)?,
			new_name,
			rename_flags,
		})
	}
}

impl fmt::Debug for RenameRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameRequest")
			.field("old_directory_id", &self.old_directory_id)
			.field("old_name", &self.old_name)
			.field("new_directory_id", &self.new_directory_id)
			.field("new_name", &self.new_name)
			.field("rename_flags", &debug::hex_u32(self.rename_flags))
			.finish()
	}
}

// }}}

// RenameResponse {{{

/// Response type for `FUSE_RENAME` and `FUSE_RENAME2`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RENAME` and `FUSE_RENAME2` operations.
pub struct RenameResponse {
	_priv: (),
}

impl RenameResponse {
	#[must_use]
	pub fn new() -> RenameResponse {
		Self { _priv: () }
	}
}

impl fmt::Debug for RenameResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameResponse").finish()
	}
}

impl server::sealed::Sealed for RenameResponse {}

impl server::FuseResponse for RenameResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}

// }}}
