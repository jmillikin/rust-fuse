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

//! Implements the `FUSE_MKNOD` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// MknodRequest {{{

/// Request type for `FUSE_MKNOD`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_MKNOD` operation.
pub struct MknodRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_mknod_in<'a>>,
	name: &'a node::Name,
}

impl MknodRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}

	#[must_use]
	pub fn mode(&self) -> node::Mode {
		node::Mode::new(self.body.as_v7p1().mode)
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
		use node::Type as T;
		let body = self.body.as_v7p1();
		match node::Type::from_mode(self.mode()) {
			Some(T::CharacterDevice | T::BlockDevice) => {
				Some(body.rdev)
			},
			_ => None,
		}
	}
}

request_try_from! { MknodRequest : fuse }

impl decode::Sealed for MknodRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for MknodRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let version_minor = request.version_minor;
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_MKNOD)?;

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
	}
}

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

// MknodResponse {{{

/// Response type for `FUSE_MKNOD`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_MKNOD` operation.
pub struct MknodResponse<'a> {
	phantom: PhantomData<&'a ()>,
	entry: node::Entry,
}

impl<'a> MknodResponse<'a> {
	#[must_use]
	pub fn new(entry: node::Entry) -> MknodResponse<'a> {
		Self {
			phantom: PhantomData,
			entry,
		}
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> &node::Entry {
		&self.entry
	}

	#[inline]
	#[must_use]
	pub fn entry_mut(&mut self) -> &mut node::Entry {
		&mut self.entry
	}
}

response_send_funcs!(MknodResponse<'_>);

impl fmt::Debug for MknodResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MknodResponse")
			.field("entry", &self.entry())
			.finish()
	}
}

impl MknodResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		if ctx.version_minor >= 9 {
			return enc.encode_sized(self.entry.as_v7p9())
		}
		enc.encode_sized(self.entry.as_v7p1())
	}
}

// }}}
