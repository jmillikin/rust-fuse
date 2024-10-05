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
use crate::server;
use crate::server::decode;
use crate::server::encode;

// GetxattrRequest {{{

/// Request type for `FUSE_GETXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_GETXATTR` operation.
pub struct GetxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_getxattr_in,
	name: &'a crate::XattrName,
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
	pub fn name(&self) -> &crate::XattrName {
		self.name
	}
}

impl server::sealed::Sealed for GetxattrRequest<'_> {}

impl<'a> server::FuseRequest<'a> for GetxattrRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_GETXATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = dec.next_sized()?;
		let name_bytes = dec.next_nul_terminated_bytes()?;
		let name = crate::XattrName::from_bytes(name_bytes.to_bytes_without_nul())?;
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
	Value(&'a crate::XattrValue),
	Size(fuse_kernel::fuse_getxattr_out),
	ErrTooBig(usize),
}

impl<'a> GetxattrResponse<'a> {
	#[inline]
	#[must_use]
	pub fn with_value(value: &'a crate::XattrValue) -> GetxattrResponse<'a> {
		GetxattrResponse {
			output: GetxattrOutput::Value(value),
		}
	}

	#[inline]
	#[must_use]
	pub fn with_value_size(value_size: usize) -> GetxattrResponse<'a> {
		if let Some(size_u32) = check_value_size(value_size) {
			let output = GetxattrOutput::Size(fuse_kernel::fuse_getxattr_out {
				size: size_u32,
				..fuse_kernel::fuse_getxattr_out::zeroed()
			});
			return GetxattrResponse { output };
		}
		GetxattrResponse {
			output: GetxattrOutput::ErrTooBig(value_size),
		}
	}
}

impl fmt::Debug for GetxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut dbg = fmt.debug_struct("GetxattrResponse");
		match self.output {
			GetxattrOutput::Value(value) => {
				dbg.field("value", &value.as_bytes());
			},
			GetxattrOutput::Size(out) => {
				dbg.field("size", &out.size);
			},
			GetxattrOutput::ErrTooBig(size) => {
				dbg.field("size", &size);
			},
		}
		dbg.finish()
	}
}

impl server::sealed::Sealed for GetxattrResponse<'_> {}

impl server::FuseResponse for GetxattrResponse<'_> {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		use GetxattrOutput as Out;
		match &self.output {
			Out::Value(value) => encode::bytes(header, value.as_bytes()),
			Out::Size(out) => encode::sized(header, out),
			Out::ErrTooBig(_) => encode::error(header, crate::Error::E2BIG),
		}
	}
}

#[inline]
#[must_use]
fn check_value_size(value_size: usize) -> Option<u32> {
	if let Some(max_len) = crate::XattrValue::MAX_LEN {
		if value_size > max_len {
			return None;
		}
	}
	u32::try_from(value_size).ok()
}

// }}}
