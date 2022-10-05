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

//! Implements the `FUSE_POLL` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugHexU32;

// PollRequest {{{

/// Request type for `FUSE_POLL`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_POLL` operation.
pub struct PollRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_poll_in,
}

impl PollRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn poll_events(&self) -> crate::PollEvents {
		self.body.events
	}

	#[must_use]
	pub fn poll_handle(&self) -> crate::PollHandle {
		crate::PollHandle { bits: self.body.kh }
	}

	#[must_use]
	pub fn flags(&self) -> PollRequestFlags {
		PollRequestFlags {
			bits: self.body.flags,
		}
	}
}

request_try_from! { PollRequest : cuse fuse }

impl decode::Sealed for PollRequest<'_> {}

impl<'a> decode::CuseRequest<'a> for PollRequest<'a> {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode(request.decoder())
	}
}

impl<'a> decode::FuseRequest<'a> for PollRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		Self::decode(request.decoder())
	}
}

impl<'a> PollRequest<'a> {
	fn decode(
		mut dec: decode::RequestDecoder<'a>,
	) -> Result<Self, server::RequestError> {
		dec.expect_opcode(fuse_kernel::FUSE_POLL)?;

		let header = dec.header();
		let body = dec.next_sized()?;
		decode::node_id(header.nodeid)?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for PollRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("PollRequest")
			.field("node_id", &self.node_id())
			.field("poll_handle", &self.poll_handle())
			.field("poll_events", &DebugHexU32(self.poll_events()))
			.field("flags", &self.flags())
			.finish()
	}
}

// }}}

// PollResponse {{{

/// Response type for `FUSE_POLL`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_COPY_FILE_RANGE` operation.
pub struct PollResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_poll_out,
}

impl<'a> PollResponse<'a> {
	#[must_use]
	pub fn new() -> PollResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_poll_out::zeroed(),
		}
	}

	#[must_use]
	pub fn poll_events(&self) -> crate::PollEvents {
		self.raw.revents
	}

	pub fn set_poll_events(&mut self, poll_events: crate::PollEvents) {
		self.raw.revents = poll_events;
	}
}

response_send_funcs!(PollResponse<'_>);

impl fmt::Debug for PollResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("PollResponse")
			.field("poll_events", &DebugHexU32(self.poll_events()))
			.finish()
	}
}

impl PollResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		enc.encode_sized(&self.raw)
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
	use crate::internal::fuse_kernel;
	bitflags!(PollRequestFlag, PollRequestFlags, u32, {
		SCHEDULE_NOTIFY = fuse_kernel::FUSE_POLL_SCHEDULE_NOTIFY;
	});
}

// }}}
