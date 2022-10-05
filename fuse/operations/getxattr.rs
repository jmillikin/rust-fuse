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

//! Implements the `FUSE_GETXATTR` operation.

use core::convert::TryFrom;
use core::fmt;
use core::num;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;
use crate::xattr;

// GetxattrRequest {{{

/// Request type for `FUSE_GETXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETXATTR` operation.
pub struct GetxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_getxattr_in,
	name: &'a xattr::Name,
}

impl GetxattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn size(&self) -> Option<num::NonZeroUsize> {
		let size = usize::try_from(self.body.size).unwrap_or(usize::MAX);
		num::NonZeroUsize::new(size)
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &xattr::Name {
		self.name
	}
}

request_try_from! { GetxattrRequest : fuse }

impl decode::Sealed for GetxattrRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for GetxattrRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_GETXATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = dec.next_sized()?;
		let name_bytes = dec.next_nul_terminated_bytes()?;
		let name = xattr::Name::from_bytes(name_bytes.to_bytes_without_nul())?;
		Ok(Self { header, body, name })
	}
}

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

// GetxattrResponse {{{

/// Response type for `FUSE_GETXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETXATTR` operation.
pub struct GetxattrResponse<'a> {
	output: GetxattrOutput<'a>,
}

enum GetxattrOutput<'a> {
	Value(&'a xattr::Value),
	Size(usize),
}

impl<'a> GetxattrResponse<'a> {
	#[inline]
	#[must_use]
	pub fn with_value(value: &'a xattr::Value) -> GetxattrResponse<'a> {
		GetxattrResponse {
			output: GetxattrOutput::Value(value),
		}
	}

	#[inline]
	#[must_use]
	pub fn with_value_size(value_size: usize) -> GetxattrResponse<'a> {
		GetxattrResponse {
			output: GetxattrOutput::Size(value_size),
		}
	}
}

response_send_funcs!(GetxattrResponse<'_>);

impl fmt::Debug for GetxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut dbg = fmt.debug_struct("GetxattrResponse");
		match self.output {
			GetxattrOutput::Value(value) => {
				dbg.field("value", &value.as_bytes());
			},
			GetxattrOutput::Size(size) => {
				dbg.field("size", &size);
			},
		}
		dbg.finish()
	}
}

impl GetxattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		match self.output {
			GetxattrOutput::Value(value) => {
				enc.encode_bytes(value.as_bytes())
			},
			GetxattrOutput::Size(size) => match check_value_size(size) {
				Ok(size_u32) => {
					enc.encode_sized(&fuse_kernel::fuse_getxattr_out {
						size: size_u32,
						..fuse_kernel::fuse_getxattr_out::zeroed()
					})
				},
				Err(err) => enc.encode_error(err),
			},
		}
	}
}

#[inline]
fn check_value_size(value_size: usize) -> Result<u32, crate::Error> {
	if let Some(max_len) = xattr::Value::MAX_LEN {
		if value_size > max_len {
			return Err(crate::Error::E2BIG);
		}
	}
	u32::try_from(value_size).map_err(|_| crate::Error::E2BIG)
}

// }}}
