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

//! Implements the `FUSE_GETATTR` operation.

use core::fmt;
use core::marker::PhantomData;
use core::slice;
use core::time::Duration;

use crate::NodeAttr;
use crate::internal::compat;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// GetattrRequest {{{

/// Request type for `FUSE_GETATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETATTR` operation.
pub struct GetattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_getattr_in<'a>>,
}

impl GetattrRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn handle(&self) -> Option<u64> {
		let body = self.body.as_v7p9()?;
		if (body.getattr_flags & fuse_kernel::FUSE_GETATTR_FH) > 0 {
			return Some(body.fh);
		}
		None
	}
}

request_try_from! { GetattrRequest : fuse }

impl decode::Sealed for GetattrRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for GetattrRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let version_minor = request.version_minor;
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_GETATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = if version_minor >= 9 {
			let body_v7p9 = dec.next_sized()?;
			compat::Versioned::new_getattr_v7p9(version_minor, body_v7p9)
		} else {
			compat::Versioned::new_getattr_v7p1(version_minor)
		};

		Ok(Self { header, body })
	}
}

impl fmt::Debug for GetattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetattrRequest")
			.field("node_id", &self.node_id())
			.field("handle", &format_args!("{:?}", &self.handle()))
			.finish()
	}
}

// }}}

// GetattrResponse {{{

/// Response type for `FUSE_GETATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETATTR` operation.
pub struct GetattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_attr_out,
}

impl<'a> GetattrResponse<'a> {
	#[must_use]
	pub fn new() -> GetattrResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_attr_out::zeroed(),
		}
	}

	#[must_use]
	pub fn attr_timeout(&self) -> Duration {
		Duration::new(self.raw.attr_valid, self.raw.attr_valid_nsec)
	}

	pub fn set_attr_timeout(&mut self, attr_timeout: Duration) {
		self.raw.attr_valid = attr_timeout.as_secs();
		self.raw.attr_valid_nsec = attr_timeout.subsec_nanos();
	}

	#[must_use]
	pub fn attr(&self) -> &NodeAttr {
		NodeAttr::new_ref(&self.raw.attr)
	}

	#[must_use]
	pub fn attr_mut(&mut self) -> &mut NodeAttr {
		NodeAttr::new_ref_mut(&mut self.raw.attr)
	}
}

response_send_funcs!(GetattrResponse<'_>);

impl fmt::Debug for GetattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetattrResponse")
			.field("attr_timeout", &self.attr_timeout())
			.field("attr", self.attr())
			.finish()
	}
}

impl GetattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);

		// The `fuse_attr::blksize` field was added in FUSE v7.9.
		if ctx.version_minor < 9 {
			let buf: &[u8] = unsafe {
				let raw_ptr = &self.raw as *const fuse_kernel::fuse_attr_out;
				slice::from_raw_parts(
					raw_ptr.cast::<u8>(),
					fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE,
				)
			};
			return enc.encode_bytes(buf);
		}

		enc.encode_sized(&self.raw)
	}
}

// }}}
