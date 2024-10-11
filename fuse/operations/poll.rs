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

use crate::internal::debug;
use crate::kernel;
use crate::server;
use crate::server::decode;

// PollHandle {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PollHandle {
	pub(crate) bits: u64,
}

impl fmt::Debug for PollHandle {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

// }}}

// PollRequest {{{

/// Request type for `FUSE_POLL`.
pub struct PollRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_poll_in,
}

impl PollRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn poll_events(&self) -> crate::PollEvents {
		self.body.events
	}

	#[must_use]
	pub fn poll_handle(&self) -> PollHandle {
		PollHandle { bits: self.body.kh }
	}

	#[must_use]
	pub fn flags(&self) -> PollRequestFlags {
		PollRequestFlags {
			bits: self.body.flags,
		}
	}
}

try_from_cuse_request!(PollRequest<'a>, |request| {
	Self::try_from(request.inner)
});

try_from_fuse_request!(PollRequest<'a>, |request| {
	Self::try_from(request.inner)
});

impl<'a> PollRequest<'a> {
	fn try_from(
		request: server::Request<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(kernel::fuse_opcode::FUSE_POLL)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}
/*
impl<'a> server::CuseRequest<'a> for PollRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::CuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode(request)
	}
}

impl<'a> server::FuseRequest<'a> for PollRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		Self::decode(request)
	}
}

*/

impl fmt::Debug for PollRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("PollRequest")
			.field("node_id", &self.node_id())
			.field("poll_handle", &self.poll_handle())
			.field("poll_events", &debug::hex_u32(self.poll_events()))
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// PollRequestFlags {{{

/// Optional flags set on [`PollRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PollRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PollRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::kernel;
	bitflags!(PollRequestFlag, PollRequestFlags, u32, {
		SCHEDULE_NOTIFY = kernel::FUSE_POLL_SCHEDULE_NOTIFY;
	});
}

// }}}
