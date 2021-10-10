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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "rename_test")]
mod rename_test;

// RenameRequest {{{

const RENAME_NOREPLACE: u32 = 1 << 0;
const RENAME_EXCHANGE: u32 = 1 << 1;
const RENAME_WHITEOUT: u32 = 1 << 2;

/// Request type for [`FuseHandlers::rename`].
///
/// [`FuseHandlers::rename`]: ../../trait.FuseHandlers.html#method.rename
pub struct RenameRequest<'a> {
	old_directory_id: NodeId,
	old_name: &'a NodeName,
	new_directory_id: NodeId,
	new_name: &'a NodeName,
	flags: RenameRequestFlags,
}

impl RenameRequest<'_> {
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

	pub fn flags(&self) -> &RenameRequestFlags {
		&self.flags
	}
}

bitflags_struct! {
	/// Optional flags set on [`RenameRequest`].
	///
	/// [`RenameRequest`]: struct.RenameRequest.html
	pub struct RenameRequestFlags(u32);

	RENAME_EXCHANGE: exchange,
	RENAME_NOREPLACE: no_replace,
	RENAME_WHITEOUT: whiteout,
}

impl fmt::Debug for RenameRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameRequest")
			.field("old_directory_id", &self.old_directory_id)
			.field("old_name", &self.old_name)
			.field("new_directory_id", &self.new_directory_id)
			.field("new_name", &self.new_name)
			.field("flags", &self.flags)
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for RenameRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		let header = buf.header();
		let mut dec = decode::RequestDecoder::new(buf);

		let mut flags = 0;
		let new_dir: u64;
		if header.opcode == fuse_kernel::FUSE_RENAME2 {
			let parsed: &fuse_kernel::fuse_rename2_in = dec.next_sized()?;
			flags = parsed.flags;
			new_dir = parsed.newdir;
		} else {
			debug_assert!(header.opcode == fuse_kernel::FUSE_RENAME);
			let parsed: &fuse_kernel::fuse_rename_in = dec.next_sized()?;
			new_dir = parsed.newdir;
		}
		let old_name = NodeName::new(dec.next_nul_terminated_bytes()?);
		let new_name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			old_directory_id: try_node_id(header.nodeid)?,
			old_name,
			new_directory_id: try_node_id(new_dir)?,
			new_name,
			flags: RenameRequestFlags::from_bits(flags),
		})
	}
}

// }}}

// RenameResponse {{{

/// Response type for [`FuseHandlers::rename`].
///
/// [`FuseHandlers::rename`]: ../../trait.FuseHandlers.html#method.rename
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

impl fmt::Debug for RenameResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RenameResponse").finish()
	}
}

impl encode::EncodeReply for RenameResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		_version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		enc.encode_header_only()
	}
}

// }}}
