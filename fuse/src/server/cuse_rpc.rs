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

use crate::io;
use crate::protocol;
use crate::protocol::cuse_init::CuseInitResponse;
use crate::server;
use crate::server::{CuseConnection, ErrorResponse, ServerError};

pub use crate::server::reply::{
	Reply,
	SendReply,
	SendResult,
	SentReply,
};

pub struct ServerContext<'a> {
	header: &'a server::RequestHeader,
	hooks: Option<&'a dyn server::ServerHooks>,
}

impl<'a> ServerContext<'a> {
	pub fn header(&self) -> &'a server::RequestHeader {
		self.header
	}
}


pub struct CuseServer<S, Handlers, Hooks> {
	socket: S,
	init_response: CuseInitResponse<'static>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, Handlers, Hooks> CuseServer<S, Handlers, Hooks> {
	fn requests(&self) -> server::CuseRequests<S> {
		server::CuseRequests::new(&self.socket, &self.init_response)
	}
}

impl<S, Handlers, Hooks> CuseServer<S, Handlers, Hooks>
where
	S: io::CuseServerSocket,
	Handlers: CuseHandlers<S>,
	Hooks: server::ServerHooks,
{
	pub fn serve(&self, buf: &mut [u8]) -> Result<(), ServerError<S::Error>> {
		let requests = self.requests();
		while let Some(request) = requests.try_next(buf)? {
			let result = cuse_request_dispatch(
				&self.socket,
				&self.handlers,
				self.hooks.as_ref(),
				request,
			);
			match result {
				Ok(()) => {},
				Err(io::ServerSendError::NotFound(_)) => {},
				Err(err) => return Err(err.into()),
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

mod internal {
	pub enum NoopServerHooks {}

	impl crate::server::ServerHooks for NoopServerHooks {}
}

impl<S, Handlers> CuseServerBuilder<S, Handlers, internal::NoopServerHooks> {
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
			socket: self.conn.socket,
			init_response: self.conn.init_response,
			handlers: self.handlers,
			hooks: self.hooks,
		}
	}
}

struct CuseReplySender<'a, S> {
	socket: &'a S,
	response_ctx: server::ResponseContext,
	sent_reply: &'a mut bool,
}

impl<S: io::CuseServerSocket> SendReply<S> for CuseReplySender<'_, S> {
	fn ok<R: Reply>(
		self,
		reply: &R,
	) -> Result<SentReply<R>, io::ServerSendError<S::Error>> {
		reply.send(self.socket, self.response_ctx)?;
		*self.sent_reply = true;
		Ok(SentReply {
			_phantom: core::marker::PhantomData,
		})
	}

	fn err<R>(
		self,
		err: impl Into<crate::Error>,
	) -> Result<SentReply<R>, io::ServerSendError<S::Error>> {
		let response = ErrorResponse::new(err.into());
		response.send(self.socket, &self.response_ctx)?;
		*self.sent_reply = true;
		Ok(SentReply {
			_phantom: core::marker::PhantomData,
		})
	}
}

fn cuse_request_dispatch<S: io::CuseServerSocket>(
	socket: &S,
	handlers: &impl CuseHandlers<S>,
	hooks: Option<&impl server::ServerHooks>,
	request: server::CuseRequest,
) -> Result<(), io::ServerSendError<S::Error>> {
	let header = request.header();
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

	let response_ctx = request.response_context();

	macro_rules! do_dispatch {
		($req_type:ty, $handler:tt) => {{
			match <$req_type>::from_cuse_request(&request) {
				Ok(request) => {
					let mut sent_reply = false;
					let reply_sender = CuseReplySender {
						socket,
						response_ctx,
						sent_reply: &mut sent_reply,
					};
					let handler_result = handlers.$handler(ctx, &request, reply_sender);
					if sent_reply {
						handler_result?;
					} else {
						let response = ErrorResponse::new(crate::Error::EIO);
						let err_result = response.send(socket, &response_ctx);
						handler_result?;
						err_result?;
					}
					Ok(())
				},
				Err(err) => {
					if let Some(ref hooks) = hooks {
						hooks.request_error(header, err);
					}
					let response = ErrorResponse::new(crate::Error::EIO);
					response.send(socket, &response_ctx)
				},
			}
		}};
	}

	use crate::Opcode as Op;
	use crate::protocol::*;
	match request.header().opcode() {
		Op::FUSE_FLUSH => do_dispatch!(FlushRequest, flush),
		Op::FUSE_FSYNC => do_dispatch!(FsyncRequest, fsync),
		#[cfg(feature = "unstable_ioctl")]
		Op::FUSE_IOCTL => do_dispatch!(IoctlRequest, ioctl),
		Op::FUSE_OPEN => do_dispatch!(OpenRequest, open),
		Op::FUSE_READ => do_dispatch!(ReadRequest, read),
		Op::FUSE_RELEASE => do_dispatch!(ReleaseRequest, release),
		Op::FUSE_WRITE => do_dispatch!(WriteRequest, write),
		_ => {
			if let Some(ref hooks) = hooks {
				let req = server::UnknownRequest::from_cuse_request(&request);
				hooks.unknown_request(&req);
			}
			let response = ErrorResponse::new(crate::Error::UNIMPLEMENTED);
			response.send(socket, &response_ctx)
		},
	}
}

fn unhandled_request<S: io::ServerSocket, R>(
	ctx: ServerContext,
	send_reply: impl SendReply<S>,
) -> Result<SentReply<R>, io::ServerSendError<S::Error>> {
	if let Some(hooks) = ctx.hooks {
		hooks.unhandled_request(ctx.header);
	}
	send_reply.err(crate::Error::UNIMPLEMENTED)
}

/// User-provided handlers for CUSE operations.
#[allow(unused_variables)]
pub trait CuseHandlers<S: io::ServerSocket> {
	fn flush(
		&self,
		ctx: ServerContext,
		request: &protocol::FlushRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FlushResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn fsync(
		&self,
		ctx: ServerContext,
		request: &protocol::FsyncRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FsyncResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	fn ioctl(
		&self,
		ctx: ServerContext,
		request: &protocol::IoctlRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::IoctlResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn open(
		&self,
		ctx: ServerContext,
		request: &protocol::OpenRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::OpenResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn read(
		&self,
		ctx: ServerContext,
		request: &protocol::ReadRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReadResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn release(
		&self,
		ctx: ServerContext,
		request: &protocol::ReleaseRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReleaseResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn write(
		&self,
		ctx: ServerContext,
		request: &protocol::WriteRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::WriteResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}
}
