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

use crate::io::{self, ServerSendError as SendError};
use crate::server::{CuseConnection, CuseRequest, Reply, ServerError};
use crate::server::basic::{
	NoopServerHooks,
	SendReply,
	SentReply,
	ServerContext,
	ServerHooks,
};
use crate::server::basic::cuse_handlers::CuseHandlers;

pub struct CuseServer<S, Handlers, Hooks> {
	conn: CuseConnection<S>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, Handlers, Hooks> CuseServer<S, Handlers, Hooks>
where
	S: io::CuseServerSocket,
	Handlers: CuseHandlers<S>,
	Hooks: ServerHooks,
{
	pub fn serve(&self, buf: &mut impl io::Buffer) -> Result<(), ServerError<S::Error>> {
		while let Some(request) = self.conn.recv(buf)? {
			let result = cuse_request_dispatch(
				&self.conn,
				&self.handlers,
				self.hooks.as_ref(),
				request,
			);
			match result {
				Ok(()) => {},
				Err(SendError::NotFound(_)) => {},
				Err(SendError::Other(err)) => return Err(ServerError::SendError(err)),
			};
		}
		Ok(())
	}
}

pub struct CuseServerBuilder<S, Handlers, Hooks> {
	conn: CuseConnection<S>,
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

struct CuseReplySender<'a, S> {
	conn: &'a CuseConnection<S>,
	request_id: u64,
	sent_reply: &'a mut bool,
}

impl<S: io::CuseServerSocket> SendReply<S> for CuseReplySender<'_, S> {
	fn ok<R: Reply>(
		self,
		reply: &R,
	) -> Result<SentReply<R>, SendError<S::Error>> {
		match self.conn.reply_ok(self.request_id, reply) {
			Ok(()) => {
				*self.sent_reply = true;
				Ok(SentReply {
					_phantom: core::marker::PhantomData,
				})
			},
			Err(err) => Err(err),
		}
	}

	fn err<R>(
		self,
		err: impl Into<crate::Error>,
	) -> Result<SentReply<R>, SendError<S::Error>> {
		match self.conn.reply_err(self.request_id, err.into()) {
			Ok(()) => {
				*self.sent_reply = true;
				Ok(SentReply {
					_phantom: core::marker::PhantomData,
				})
			},
			Err(err) => Err(err),
		}
	}
}

fn cuse_request_dispatch<S: io::CuseServerSocket>(
	conn: &CuseConnection<S>,
	handlers: &impl CuseHandlers<S>,
	hooks: Option<&impl ServerHooks>,
	request: CuseRequest,
) -> Result<(), SendError<S::Error>> {
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

	macro_rules! do_dispatch {
		($req_type:ty, $handler:tt) => {{
			match <$req_type>::from_cuse_request(&request) {
				Ok(request) => {
					let mut sent_reply = false;
					let reply_sender = CuseReplySender {
						conn,
						request_id,
						sent_reply: &mut sent_reply,
					};
					let handler_result = handlers.$handler(ctx, &request, reply_sender);
					if sent_reply {
						handler_result?;
					} else {
						let err_result = conn.reply_err(request_id, crate::Error::EIO);
						handler_result?;
						err_result?;
					}
					Ok(())
				},
				Err(err) => {
					if let Some(ref hooks) = hooks {
						hooks.request_error(header, err);
					}
					conn.reply_err(request_id, crate::Error::EIO)
				},
			}
		}};
	}

	use crate::server::CuseOperation as CuseOp;
	use crate::protocol::*;
	match request.operation() {
		Some(CuseOp::Flush) => do_dispatch!(FlushRequest, flush),
		Some(CuseOp::Fsync) => do_dispatch!(FsyncRequest, fsync),
		#[cfg(feature = "unstable_ioctl")]
		Some(CuseOp::Ioctl) => do_dispatch!(IoctlRequest, ioctl),
		Some(CuseOp::Open) => do_dispatch!(OpenRequest, open),
		Some(CuseOp::Read) => do_dispatch!(ReadRequest, read),
		Some(CuseOp::Release) => do_dispatch!(ReleaseRequest, release),
		Some(CuseOp::Write) => do_dispatch!(WriteRequest, write),
		_ => {
			if let Some(ref hooks) = hooks {
				let request = request.into_unknown();
				hooks.unknown_request(&request);
			}
			conn.reply_err(request_id, crate::Error::UNIMPLEMENTED)
		},
	}
}
