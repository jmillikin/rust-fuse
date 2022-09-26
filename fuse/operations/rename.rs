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
use core::marker::PhantomData;

use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;

// RenameRequest {{{

/// Request type for `FUSE_RENAME` and `FUSE_RENAME2`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RENAME` and `FUSE_RENAME2` operations.
pub struct RenameRequest<'a> {
	old_directory_id: NodeId,
	old_name: &'a NodeName,
	new_directory_id: NodeId,
	new_name: &'a NodeName,
	rename_flags: u32,
}

impl<'a> RenameRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
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
		let old_name = NodeName::new(dec.next_nul_terminated_bytes()?);
		let new_name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			old_directory_id: decode::node_id(header.nodeid)?,
			old_name,
			new_directory_id: decode::node_id(new_dir)?,
			new_name,
			rename_flags,
		})
	}

	pub fn old_directory_id(&self) -> NodeId {
		self.old_directory_id
	}

	pub fn old_name(&self) -> &NodeName {
		self.old_name
	}

	pub fn new_directory_id(&self) -> NodeId {
		self.new_directory_id
	}

	pub fn new_name(&self) -> &NodeName {
		self.new_name
	}

	pub fn rename_flags(&self) -> crate::RenameFlags {
		self.rename_flags
	}
}

impl fmt::Debug for RenameRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameRequest")
			.field("old_directory_id", &self.old_directory_id)
			.field("old_name", &self.old_name)
			.field("new_directory_id", &self.new_directory_id)
			.field("new_name", &self.new_name)
			.field("rename_flags", &DebugHexU32(self.rename_flags))
			.finish()
	}
}

// }}}

// RenameResponse {{{

/// Response type for `FUSE_RENAME` and `FUSE_RENAME2`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_RENAME` and `FUSE_RENAME2` operations.
pub struct RenameResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> RenameResponse<'a> {
	pub fn new() -> RenameResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(RenameResponse<'_>);

impl fmt::Debug for RenameResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameResponse").finish()
	}
}

impl RenameResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_header_only()
	}
}

// }}}
