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

use core::fmt;
use core::num;

use crate::kernel;
use crate::server;

// InterruptRequest {{{

/// Request type for `FUSE_INTERRUPT`.
pub struct InterruptRequest<'a> {
	body: &'a kernel::fuse_interrupt_in,
}

impl InterruptRequest<'_> {
	#[must_use]
	pub fn request_id(&self) -> num::NonZeroU64 {
		unsafe { num::NonZeroU64::new_unchecked(self.body.unique) }
	}
}

try_from_cuse_request!(InterruptRequest<'a>, |request| {
	Self::try_from(request.inner)
});

try_from_fuse_request!(InterruptRequest<'a>, |request| {
	Self::try_from(request.inner)
});

impl<'a> InterruptRequest<'a> {
	fn try_from(
		request: server::Request<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_INTERRUPT)?;
		let body: &kernel::fuse_interrupt_in = dec.next_sized()?;
		if body.unique == 0 {
			return Err(server::RequestError::MissingRequestId);
		}
		Ok(Self { body })
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
