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
use crate::server::io;
use crate::server::io::encode;

// SyncfsRequest {{{

/// Request type for `FUSE_SYNCFS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SYNCFS` operation.
pub struct SyncfsRequest<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> SyncfsRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
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

response_send_funcs!(SyncfsResponse<'_>);

impl fmt::Debug for SyncfsResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SyncfsResponse").finish()
	}
}

impl SyncfsResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_header_only()
	}
}
