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
use core::num;

use crate::kernel;
use crate::server::decode;

// GetxattrRequest {{{

/// Request type for `FUSE_GETXATTR`.
pub struct GetxattrRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_getxattr_in,
	name: &'a core::ffi::CStr,
}

impl GetxattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn size(&self) -> Option<num::NonZeroUsize> {
		let size = usize::try_from(self.body.size).unwrap_or(usize::MAX);
		num::NonZeroUsize::new(size)
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &core::ffi::CStr {
		self.name
	}
}

try_from_fuse_request!(GetxattrRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_GETXATTR)?;

	let header = dec.header();
	decode::node_id(header.nodeid)?;

	let body = dec.next_sized()?;
	let name = dec.next_cstr()?;
	Ok(Self { header, body, name })
});

impl fmt::Debug for GetxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetxattrRequest")
			.field("node_id", &self.node_id())
			.field("size", &format_args!("{:?}", &self.size()))
			.field("name", &self.name())
			.finish()
	}
}

// }}}
