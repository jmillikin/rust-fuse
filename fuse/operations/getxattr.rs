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

use crate::NodeId;
use crate::XattrError;
use crate::XattrName;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugBytesAsString;

#[cfg(rust_fuse_test = "getxattr_test")]
mod getxattr_test;

// GetxattrRequest {{{

/// Request type for [`FuseHandlers::getxattr`].
///
/// [`FuseHandlers::getxattr`]: ../../trait.FuseHandlers.html#method.getxattr
pub struct GetxattrRequest<'a> {
	node_id: NodeId,
	size: Option<num::NonZeroU32>,
	name: &'a XattrName,
}

impl<'a> GetxattrRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_GETXATTR)?;

		let raw: &'a fuse_kernel::fuse_getxattr_in = dec.next_sized()?;
		Ok(Self {
			node_id: decode::node_id(dec.header().nodeid)?,
			size: num::NonZeroU32::new(raw.size),
			name: XattrName::new(dec.next_nul_terminated_bytes()?),
		})
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn size(&self) -> Option<num::NonZeroU32> {
		self.size
	}

	pub fn name(&self) -> &XattrName {
		self.name
	}
}

impl fmt::Debug for GetxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("GetxattrRequest")
			.field("node_id", &self.node_id)
			.field("size", &format_args!("{:?}", &self.size))
			.field("name", &self.name)
			.finish()
	}
}

// }}}

// GetxattrResponse {{{

/// Response type for [`FuseHandlers::getxattr`].
///
/// [`FuseHandlers::getxattr`]: ../../trait.FuseHandlers.html#method.getxattr
pub struct GetxattrResponse<'a> {
	request_size: Option<num::NonZeroU32>,
	raw: fuse_kernel::fuse_getxattr_out,
	value: &'a [u8],
}

impl<'a> GetxattrResponse<'a> {
	pub fn new(request_size: Option<num::NonZeroU32>) -> GetxattrResponse<'a> {
		Self {
			request_size,
			raw: fuse_kernel::fuse_getxattr_out::zeroed(),
			value: &[],
		}
	}

	pub fn request_size(&self) -> Option<num::NonZeroU32> {
		self.request_size
	}

	pub fn value(&self) -> &[u8] {
		self.value
	}

	pub fn set_value(&mut self, value: &'a [u8]) {
		self.try_set_value(value).unwrap()
	}

	pub fn try_set_value(&mut self, value: &'a [u8]) -> Result<(), XattrError> {
		if value.len() > crate::XATTR_SIZE_MAX {
			return Err(XattrError::exceeds_size_max(value.len()));
		}
		let value_len = value.len() as u32;

		match self.request_size {
			None => {
				self.raw.size = value_len;
			},
			Some(request_size) => {
				if value_len > request_size.get() {
					return Err(XattrError::exceeds_request_size(
						value.len(),
						request_size.get(),
					));
				}
				self.value = value;
			},
		}
		return Ok(());
	}

	response_send_funcs!();
}

impl fmt::Debug for GetxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut out = fmt.debug_struct("GetxattrResponse");
		out.field("request_size", &format_args!("{:?}", &self.request_size));
		if self.request_size.is_some() {
			out.field("value", &DebugBytesAsString(&self.value));
		}
		out.finish()
	}
}

impl GetxattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		if self.raw.size != 0 {
			enc.encode_sized(&self.raw)
		} else {
			enc.encode_bytes(&self.value)
		}
	}
}

// }}}
