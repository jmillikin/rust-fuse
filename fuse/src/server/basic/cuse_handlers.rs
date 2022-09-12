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

use crate::io::{OutputStream, SendError};
use crate::protocol;
use crate::server::basic::server::{self, SendReply, SentReply};

/// User-provided handlers for CUSE operations.
#[allow(unused_variables)]
pub trait CuseHandlers<S: OutputStream> {
	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FlushResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FsyncResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::IoctlResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::OpenResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReadResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReleaseResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::WriteResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}
}
