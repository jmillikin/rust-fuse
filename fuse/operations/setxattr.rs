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

use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;
use crate::xattr;

use crate::protocol::common::DebugHexU32;

// SetxattrRequest {{{

/// Request type for `FUSE_SETXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETXATTR` operation.
pub struct SetxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_setxattr_in<'a>>,
	name: &'a xattr::Name,
	value: &'a xattr::Value,
}

impl SetxattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &xattr::Name {
		self.name
	}

	#[inline]
	#[must_use]
	pub fn flags(&self) -> SetxattrRequestFlags {
		if let Some(body) = self.body.as_v7p33() {
			return SetxattrRequestFlags {
				bits: body.setxattr_flags,
			}
		}
		SetxattrRequestFlags::new()
	}

	#[inline]
	#[must_use]
	pub fn setxattr_flags(&self) -> crate::SetxattrFlags {
		self.body.as_v7p1().flags
	}

	#[inline]
	#[must_use]
	pub fn value(&self) -> &xattr::Value {
		self.value
	}
}

request_try_from! { SetxattrRequest : fuse }

impl decode::Sealed for SetxattrRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for SetxattrRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_SETXATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = if request.have_setxattr_ext() {
			let body_v7p33 = dec.next_sized()?;
			compat::Versioned::new_setxattr_v7p33(body_v7p33)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_setxattr_v7p1(body_v7p1)
		};

		let name_bytes = dec.next_nul_terminated_bytes()?;
		let name = xattr::Name::from_bytes(name_bytes.to_bytes_without_nul())?;
		let value_bytes = dec.next_bytes(body.as_v7p1().size)?;
		let value = xattr::Value::new(value_bytes)?;

		Ok(Self { header, body, name, value })
	}
}

impl fmt::Debug for SetxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetxattrRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.field("flags", &self.flags())
			.field("setxattr_flags", &DebugHexU32(self.setxattr_flags()))
			.field("value", &self.value().as_bytes())
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
	#[inline]
	#[must_use]
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
