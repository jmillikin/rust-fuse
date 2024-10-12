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

use crate::internal::compat;
use crate::internal::debug;
use crate::kernel;
use crate::server::decode;

// SetxattrRequest {{{

/// Request type for `FUSE_SETXATTR`.
pub struct SetxattrRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_setxattr_in<'a>>,
	name: &'a core::ffi::CStr,
	value: &'a [u8],
}

impl SetxattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &core::ffi::CStr {
		self.name
	}

	#[inline]
	#[must_use]
	pub fn flags(&self) -> SetxattrRequestFlags {
		if let Some(body) = self.body.as_v7p33() {
			return SetxattrRequestFlags {
				bits: body.setxattr_flags,
			};
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
	pub fn value(&self) -> &[u8] {
		self.value
	}
}

try_from_fuse_request!(SetxattrRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_SETXATTR)?;

	let header = dec.header();
	decode::node_id(header.nodeid)?;

	let body = if request.layout.have_setxattr_ext() {
		let body_v7p33 = dec.next_sized()?;
		compat::Versioned::new_setxattr_v7p33(body_v7p33)
	} else {
		let body_v7p1 = dec.next_sized()?;
		compat::Versioned::new_setxattr_v7p1(body_v7p1)
	};

	let name = dec.next_cstr()?;
	let value = dec.next_bytes(body.as_v7p1().size)?;

	Ok(Self { header, body, name, value })
});

impl fmt::Debug for SetxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetxattrRequest")
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.field("flags", &self.flags())
			.field("setxattr_flags", &debug::hex_u32(self.setxattr_flags()))
			.field("value", &debug::bytes(self.value))
			.finish()
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
	use crate::kernel;
	bitflags!(SetxattrRequestFlag, SetxattrRequestFlags, u32, {
		ACL_KILL_SGID = kernel::FUSE_SETXATTR_ACL_KILL_SGID;
	});
}

// }}}
