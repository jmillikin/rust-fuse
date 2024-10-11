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
use crate::kernel;
use crate::server::decode;

// MknodRequest {{{

/// Request type for `FUSE_MKNOD`.
pub struct MknodRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_mknod_in<'a>>,
	name: &'a crate::NodeName,
}

impl MknodRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
		self.name
	}

	#[must_use]
	pub fn mode(&self) -> crate::FileMode {
		crate::FileMode::new(self.body.as_v7p1().mode)
	}

	#[must_use]
	pub fn umask(&self) -> u32 {
		if let Some(body) = self.body.as_v7p12() {
			return body.umask;
		}
		0
	}

	#[must_use]
	pub fn device_number(&self) -> Option<u32> {
		use crate::FileType as T;
		let body = self.body.as_v7p1();
		match crate::FileType::from_mode(self.mode()) {
			Some(T::CharacterDevice | T::BlockDevice) => Some(body.rdev),
			_ => None,
		}
	}
}

try_from_fuse_request!(MknodRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_MKNOD)?;

	let header = dec.header();
	decode::node_id(dec.header().nodeid)?;

	let body = if version_minor >= 12 {
		let body_v7p12 = dec.next_sized()?;
		compat::Versioned::new_mknod_v7p12(version_minor, body_v7p12)
	} else {
		let body_v7p1 = dec.next_sized()?;
		compat::Versioned::new_mknod_v7p1(version_minor, body_v7p1)
	};

	let name = dec.next_node_name()?;

	Ok(Self { header, body, name })
});

impl fmt::Debug for MknodRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MknodRequest")
			.field("parent_id", &self.parent_id())
			.field("name", &self.name())
			.field("mode", &self.mode())
			.field("umask", &format_args!("{:#o}", &self.umask()))
			.field("device_number", &format_args!("{:?}", self.device_number()))
			.finish()
	}
}

// }}}
