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

#[cfg(rust_fuse_test = "setxattr_test")]
mod setxattr_test;

// SetxattrRequest {{{

const XATTR_CREATE: u32 = 1 << 0;
const XATTR_REPLACE: u32 = 1 << 1;

pub struct SetxattrRequest<'a> {
	node_id: NodeId,
	flags: SetxattrRequestFlags,
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
		let raw: &'a fuse_setxattr_in_v7p1 = dec.next_sized()?;
		let name = XattrName::new(dec.next_nul_terminated_bytes()?);
		let value = dec.next_bytes(raw.size)?;
		Ok(Self {
			node_id: decode::node_id(dec.header().nodeid)?,
			flags: SetxattrRequestFlags::from_bits(raw.flags),
			name,
			value,
		})
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn name(&self) -> &XattrName {
		self.name
	}

	pub fn flags(&self) -> &SetxattrRequestFlags {
		&self.flags
	}

	pub fn value(&self) -> &[u8] {
		self.value
	}
}

bitflags_struct! {
	/// Optional flags set on [`SetxattrRequest`].
	///
	/// [`SetxattrRequest`]: struct.SetxattrRequest.html
	pub struct SetxattrRequestFlags(u32);

	XATTR_CREATE: create,
	XATTR_REPLACE: replace,
}

impl fmt::Debug for SetxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetxattrRequest")
			.field("node_id", &self.node_id)
			.field("name", &self.name)
			.field("flags", &self.flags)
			.field("value", &DebugBytesAsString(&self.value))
			.finish()
	}
}

// }}}

// SetxattrResponse {{{

pub struct SetxattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SetxattrResponse<'a> {
	pub fn new() -> SetxattrResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}

	response_send_funcs!();
}

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
