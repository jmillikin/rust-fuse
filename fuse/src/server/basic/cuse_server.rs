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
use crate::io::{self, Error, OutputStream};
use crate::server::{CuseConnection, CuseRequest, Reply};
use crate::server::basic::{
	NoopServerHooks,
	SendReply,
	ServerContext,
	ServerHooks,
};
use crate::server::basic::cuse_handlers::CuseHandlers;

pub struct CuseServer<Stream, Handlers, Hooks> {
	conn: CuseConnection<Stream>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, E, Handlers, Hooks> CuseServer<S, Handlers, Hooks>
where
	S: io::InputStream<Error = E> + io::OutputStream<Error = E>,
	Handlers: CuseHandlers<S>,
	Hooks: ServerHooks,
{
	pub fn serve(&self, buf: &mut impl io::Buffer) -> Result<(), io::Error<E>> {
		while let Some(request) = self.conn.recv(buf)? {
			cuse_request_dispatch(
				&self.conn,
				&self.handlers,
				self.hooks.as_ref(),
				request,
			)?;
		}
		Ok(())
	}
}

pub struct CuseServerBuilder<Stream, Handlers, Hooks> {
	conn: CuseConnection<Stream>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, Handlers> CuseServerBuilder<S, Handlers, NoopServerHooks> {
	pub fn new(conn: CuseConnection<S>, handlers: Handlers) -> Self {
		Self {
			conn,
			handlers,
			hooks: None,
		}
	}
}

impl<S, Handlers, Hooks> CuseServerBuilder<S, Handlers, Hooks> {
	pub fn server_hooks<H>(
		self,
		hooks: H,
	) -> CuseServerBuilder<S, Handlers, H> {
		CuseServerBuilder {
			conn: self.conn,
			handlers: self.handlers,
			hooks: Some(hooks),
		}
	}

	pub fn build(self) -> CuseServer<S, Handlers, Hooks> {
		CuseServer {
			conn: self.conn,
			handlers: self.handlers,
			hooks: self.hooks,
		}
	}
}

pub(super) struct CuseReplySender<'a, S> {
	pub(super) conn: &'a CuseConnection<S>,
	pub(super) request_id: u64,
}

impl<S: OutputStream, R: Reply> SendReply<S, R> for CuseReplySender<'_, S> {
	fn ok(self, reply: &R) -> Result<(), Error<S::Error>> {
		self.conn.reply_ok(self.request_id, reply)
	}

	fn err(self, err: impl Into<NonZeroU16>) -> Result<(), Error<S::Error>> {
		self.conn.reply_err(self.request_id, err.into())
	}
}

fn cuse_request_dispatch<S: OutputStream>(
	conn: &CuseConnection<S>,
	handlers: &impl CuseHandlers<S>,
	hooks: Option<&impl ServerHooks>,
	request: CuseRequest,
) -> Result<(), io::Error<S::Error>> {
	let header = request.header();
	let request_id = header.request_id();
	if let Some(hooks) = hooks {
		hooks.request(header);
	}

	let ctx = ServerContext {
		header,
		hooks: match hooks {
			None => None,
			Some(x) => Some(x),
		},
	};

	#[rustfmt::skip]
	macro_rules! do_dispatch {
		($handler:tt) => {{
			match request.decode() {
				Ok(request) => {
					let reply_sender = CuseReplySender { conn, request_id };
					handlers.$handler(ctx, &request, reply_sender)
				},
				Err(err) => {
					if let Some(ref hooks) = hooks {
						hooks.request_error(header, err);
					}
					conn.reply_err(request_id, ErrorCode::EIO.into())
				},
			}
		}};
	}

	use crate::server::CuseOperation as CuseOp;
	match request.operation() {
		Some(CuseOp::Flush) => do_dispatch!(flush),
		Some(CuseOp::Fsync) => do_dispatch!(fsync),
		#[cfg(feature = "unstable_ioctl")]
		Some(CuseOp::Ioctl) => do_dispatch!(ioctl),
		Some(CuseOp::Open) => do_dispatch!(open),
		Some(CuseOp::Read) => do_dispatch!(read),
		Some(CuseOp::Release) => do_dispatch!(release),
		Some(CuseOp::Write) => do_dispatch!(write),
		_ => {
			if let Some(ref hooks) = hooks {
				let request = request.into_unknown();
				hooks.unknown_request(&request);
			}
			conn.reply_err(request_id, ErrorCode::ENOSYS.into())
		},
	}
}
