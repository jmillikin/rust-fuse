// Copyright 2021 John Millikin and the rust-fuse contributors.
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

use core::num::NonZeroU16;

use crate::error::ErrorCode;
use crate::io::{Error, OutputStream};
use crate::server::RequestHeader;
use crate::server::basic::server_hooks::ServerHooks;

pub struct ServerContext<'a> {
	pub(super) header: &'a RequestHeader,
	pub(super) hooks: Option<&'a dyn ServerHooks>,
}

impl<'a> ServerContext<'a> {
	pub fn header(&self) -> &'a RequestHeader {
		self.header
	}
}

pub trait SendReply<S: OutputStream, R> {
	fn ok(self, reply: &R) -> Result<(), Error<S::Error>>;
	fn err(self, err: impl Into<NonZeroU16>) -> Result<(), Error<S::Error>>;
}

pub(super) fn unhandled_request<S: OutputStream, R>(
	ctx: ServerContext,
	send_reply: impl SendReply<S, R>,
) {
	if let Some(hooks) = ctx.hooks {
		hooks.unhandled_request(ctx.header);
	}
	let _ = send_reply.err(ErrorCode::ENOSYS);
}
