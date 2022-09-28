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

//! Implements the `FUSE_DESTROY` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::encode;

// DestroyRequest {{{

/// Request type for `FUSE_DESTROY`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_DESTROY` operation.
pub struct DestroyRequest<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> DestroyRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_DESTROY)?;
		Ok(Self {
			phantom: PhantomData,
		})
	}
}

impl fmt::Debug for DestroyRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("DestroyRequest").finish()
	}
}

// }}}

// DestroyResponse {{{

/// Response type for `FUSE_DESTROY`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_DESTROY` operation.
pub struct DestroyResponse<'a> {
	phantom: PhantomData<&'a ()>,
}

impl<'a> DestroyResponse<'a> {
	pub fn new() -> DestroyResponse<'a> {
		Self {
			phantom: PhantomData,
		}
	}
}

response_send_funcs!(DestroyResponse<'_>);

impl fmt::Debug for DestroyResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("DestroyResponse").finish()
	}
}

impl DestroyResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_header_only()
	}
}

// }}}
