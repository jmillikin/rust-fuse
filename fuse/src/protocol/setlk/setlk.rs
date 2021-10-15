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

use crate::protocol::common::file_lock::{Lock, LockRange, F_UNLCK};
use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "setlk_test")]
mod setlk_test;

// SetlkRequest {{{

/// Request type for [`FuseHandlers::setlk`].
///
/// [`FuseHandlers::setlk`]: ../../trait.FuseHandlers.html#method.setlk
pub struct SetlkRequest<'a> {
	raw: &'a fuse_kernel::fuse_lk_in,
	node_id: NodeId,
	command: SetlkCommand,
}

impl SetlkRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn owner(&self) -> u64 {
		self.raw.owner
	}

	pub fn command(&self) -> &SetlkCommand {
		&self.command
	}

	pub fn flags(&self) -> SetlkRequestFlags {
		SetlkRequestFlags::from_bits(self.raw.lk_flags)
	}
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SetlkCommand {
	SetLock(Lock),
	TrySetLock(Lock),

	#[non_exhaustive]
	ClearLocks {
		range: LockRange,
		process_id: u32,
	},
}

bitflags_struct! {
	/// Optional flags set on [`SetlkRequest`].
	///
	/// [`SetlkRequest`]: struct.SetlkRequest.html
	pub struct SetlkRequestFlags(u32);

	fuse_kernel::FUSE_LK_FLOCK: flock,
}

impl fmt::Debug for SetlkRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetlkRequest")
			.field("node_id", &self.node_id)
			.field("handle", &self.raw.fh)
			.field("owner", &self.raw.owner)
			.field("command", &self.command)
			.field("flags", &self.flags())
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for SetlkRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::RequestError> {
		let header = buf.header();

		let is_setlkw: bool;
		if header.opcode == fuse_kernel::FUSE_SETLKW {
			is_setlkw = true;
		} else {
			buf.expect_opcode(fuse_kernel::FUSE_SETLK)?;
			is_setlkw = false;
		}

		let mut dec = decode::RequestDecoder::new(buf);
		let raw: &fuse_kernel::fuse_lk_in = dec.next_sized()?;
		let node_id = try_node_id(header.nodeid)?;
		let command = parse_setlk_cmd(is_setlkw, &raw.lk)?;

		Ok(Self {
			raw,
			node_id,
			command,
		})
	}
}

fn parse_setlk_cmd(
	is_setlkw: bool,
	raw: &fuse_kernel::fuse_file_lock,
) -> Result<SetlkCommand, io::RequestError> {
	if raw.r#type == F_UNLCK {
		return Ok(SetlkCommand::ClearLocks {
			range: LockRange::parse(*raw),
			process_id: raw.pid,
		});
	}

	match Lock::parse(*raw) {
		Some(lock) => Ok(if is_setlkw {
			SetlkCommand::SetLock(lock)
		} else {
			SetlkCommand::TrySetLock(lock)
		}),
		None => return Err(io::RequestError::InvalidLockType),
	}
}

// }}}

// SetlkResponse {{{

/// Response type for [`FuseHandlers::setlk`].
///
/// [`FuseHandlers::setlk`]: ../../trait.FuseHandlers.html#method.setlk
pub struct SetlkResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SetlkResponse<'a> {
	pub fn new() -> SetlkResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for SetlkResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetlkResponse").finish()
	}
}

impl encode::EncodeReply for SetlkResponse<'_> {
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
