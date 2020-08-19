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

use crate::protocol::prelude::*;

#[cfg(test)]
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

impl GetxattrRequest<'_> {
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

impl<'a> fuse_io::DecodeRequest<'a> for GetxattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_GETXATTR);

		let raw: &'a fuse_kernel::fuse_getxattr_in = dec.next_sized()?;
		Ok(Self {
			node_id: try_node_id(header.nodeid)?,
			size: num::NonZeroU32::new(raw.size),
			name: XattrName::new(dec.next_nul_terminated_bytes()?),
		})
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
			raw: Default::default(),
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

	pub fn try_set_value(&mut self, value: &'a [u8]) -> Option<()> {
		// TODO: Result
		if value.len() > crate::XATTR_SIZE_MAX {
			return None; // ERANGE
		}
		let value_len = value.len() as u32;

		match self.request_size {
			None => {
				self.raw.size = value_len;
			},
			Some(request_size) => {
				if value_len > request_size.get() {
					return None; // ERANGE
				}
				self.value = value;
			},
		}
		return Some(());
	}
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

impl fuse_io::EncodeResponse for GetxattrResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		if self.raw.size != 0 {
			enc.encode_sized(&self.raw)
		} else {
			enc.encode_bytes(&self.value)
		}
	}
}

// }}}
