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

//! Implements the `FUSE_SETXATTR` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::NodeId;
use crate::XattrName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugBytesAsString;
use crate::protocol::common::DebugHexU32;

// SetxattrRequest {{{

/// Request type for `FUSE_SETXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETXATTR` operation.
pub struct SetxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: fuse_kernel::fuse_setxattr_in,
	name: &'a XattrName,
	value: &'a [u8],
}

#[repr(C)]
pub(crate) struct fuse_setxattr_in_v7p1 {
	pub size:  u32,
	pub flags: u32,
}

impl<'a> SetxattrRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_SETXATTR)?;

		let header = dec.header();
		let mut raw = fuse_kernel::fuse_setxattr_in::zeroed();
		if request.have_setxattr_ext() {
			raw = *(dec.next_sized()?);
		} else {
			let old_raw: &fuse_setxattr_in_v7p1 = dec.next_sized()?;
			raw.size = old_raw.size;
			raw.flags = old_raw.flags;
		}

		decode::node_id(header.nodeid)?;
		let name = XattrName::new(dec.next_nul_terminated_bytes()?);
		let value = dec.next_bytes(raw.size)?;
		Ok(Self { header, raw, name, value })
	}

	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.header.nodeid) }
	}

	pub fn name(&self) -> &XattrName {
		self.name
	}

	pub fn flags(&self) -> SetxattrRequestFlags {
		SetxattrRequestFlags {
			bits: self.raw.setxattr_flags,
		}
	}

	pub fn setxattr_flags(&self) -> crate::SetxattrFlags {
		self.raw.flags
	}

	pub fn value(&self) -> &[u8] {
		self.value
	}
}

impl fmt::Debug for SetxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetxattrRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.field("flags", &self.flags())
			.field("setxattr_flags", &DebugHexU32(self.setxattr_flags()))
			.field("value", &DebugBytesAsString(self.value))
			.finish()
	}
}

// }}}

// SetxattrResponse {{{

/// Response type for `FUSE_SETXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETXATTR` operation.
pub struct SetxattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SetxattrResponse<'a> {
	pub fn new() -> SetxattrResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(SetxattrResponse<'_>);

impl fmt::Debug for SetxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetxattrResponse").finish()
	}
}

impl SetxattrResponse<'_> {
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

// SetxattrRequestFlags {{{

/// Optional flags set on [`SetxattrRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SetxattrRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SetxattrRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(SetxattrRequestFlag, SetxattrRequestFlags, u32, {
		ACL_KILL_SGID = fuse_kernel::FUSE_SETXATTR_ACL_KILL_SGID;
	});
}

// }}}
