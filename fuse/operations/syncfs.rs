// Copyright 2022 John Millikin and the rust-fuse contributors.
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

//! Implements the `FUSE_SYNCFS` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::encode;

// SyncfsRequest {{{

/// Request type for `FUSE_SYNCFS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYNCFS` operation.
pub struct SyncfsRequest<'a> {
	phantom: PhantomData<&'a ()>,
}

impl server::sealed::Sealed for SyncfsRequest<'_> {}

impl<'a> server::FuseRequest<'a> for SyncfsRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_SYNCFS)?;

		let _body: &'a fuse_kernel::fuse_syncfs_in = dec.next_sized()?;
		Ok(SyncfsRequest {
			phantom: PhantomData,
		})
	}
}

impl fmt::Debug for SyncfsRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SyncfsRequest").finish()
	}
}

// }}}

// SyncfsResponse {{{

/// Response type for `FUSE_SYNCFS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYNCFS` operation.
pub struct SyncfsResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SyncfsResponse<'a> {
	#[must_use]
	pub fn new() -> SyncfsResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

impl fmt::Debug for SyncfsResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SyncfsResponse").finish()
	}
}

impl server::sealed::Sealed for SyncfsResponse<'_> {}

impl server::FuseResponse for SyncfsResponse<'_> {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		_options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		encode::header_only(header)
	}
}
