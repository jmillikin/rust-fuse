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

//! Implements the `FUSE_INTERRUPT` operation.

use core::fmt;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;

// InterruptRequest {{{

/// Request type for `FUSE_INTERRUPT`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_INTERRUPT` operation.
pub struct InterruptRequest<'a> {
	body: &'a fuse_kernel::fuse_interrupt_in,
}

impl<'a> InterruptRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_INTERRUPT)?;
		let body = dec.next_sized()?;
		Ok(Self { body })
	}

	pub fn request_id(&self) -> u64 {
		self.body.unique
	}
}

impl fmt::Debug for InterruptRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("InterruptRequest")
			.field("request_id", &self.request_id())
			.finish()
	}
}

// }}}
