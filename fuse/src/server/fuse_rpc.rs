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
use crate::protocol::fuse_init::FuseInitResponse;
use crate::server;
use crate::server::{ErrorResponse, FuseConnection, ServerError};

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

fn unhandled_request<S: io::ServerSocket, R>(
	ctx: ServerContext,
	send_reply: impl SendReply<S>,
) -> Result<SentReply<R>, io::ServerSendError<S::Error>> {
	if let Some(hooks) = ctx.hooks {
		hooks.unhandled_request(ctx.header);
	}
	send_reply.err(crate::Error::UNIMPLEMENTED)
}

pub struct FuseServer<S, Handlers, Hooks> {
	socket: S,
	init_response: FuseInitResponse,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, Handlers, Hooks> FuseServer<S, Handlers, Hooks> {
	fn requests(&self) -> server::FuseRequests<S> {
		server::FuseRequests::new(&self.socket, &self.init_response)
	}
}

impl<S, Handlers, Hooks> FuseServer<S, Handlers, Hooks>
where
	S: io::FuseServerSocket,
	Handlers: FuseHandlers<S>,
	Hooks: server::ServerHooks,
{
	pub fn serve(&self, buf: &mut [u8]) -> Result<(), ServerError<S::Error>> {
		let requests = self.requests();
		while let Some(request) = requests.try_next(buf)? {
			let result = fuse_request_dispatch(
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

pub struct FuseServerBuilder<S, Handlers, Hooks> {
	conn: FuseConnection<S>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

mod internal {
	pub enum NoopServerHooks {}

	impl crate::server::ServerHooks for NoopServerHooks {}
}

impl<S, Handlers> FuseServerBuilder<S, Handlers, internal::NoopServerHooks> {
	pub fn new(conn: FuseConnection<S>, handlers: Handlers) -> Self {
		Self {
			conn,
			handlers,
			hooks: None,
		}
	}
}

impl<S, Handlers, Hooks> FuseServerBuilder<S, Handlers, Hooks> {
	pub fn server_hooks<H>(
		self,
		hooks: H,
	) -> FuseServerBuilder<S, Handlers, H> {
		FuseServerBuilder {
			conn: self.conn,
			handlers: self.handlers,
			hooks: Some(hooks),
		}
	}

	pub fn build(self) -> FuseServer<S, Handlers, Hooks> {
		FuseServer {
			socket: self.conn.socket,
			init_response: self.conn.init_response,
			handlers: self.handlers,
			hooks: self.hooks,
		}
	}
}

struct FuseReplySender<'a, S> {
	socket: &'a S,
	response_ctx: server::ResponseContext,
	sent_reply: &'a mut bool,
}

impl<S: io::FuseServerSocket> SendReply<S> for FuseReplySender<'_, S> {
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

fn fuse_request_dispatch<S: io::FuseServerSocket>(
	socket: &S,
	handlers: &impl FuseHandlers<S>,
	hooks: Option<&impl server::ServerHooks>,
	request: server::FuseRequest,
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
			match <$req_type>::from_fuse_request(&request) {
				Ok(request) => {
					let mut sent_reply = false;
					let reply_sender = FuseReplySender {
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
		Op::FUSE_ACCESS => do_dispatch!(AccessRequest, access),
		#[cfg(feature = "unstable_bmap")]
		Op::FUSE_BMAP => do_dispatch!(BmapRequest, bmap),
		Op::FUSE_CREATE => do_dispatch!(CreateRequest, create),
		Op::FUSE_FALLOCATE => do_dispatch!(FallocateRequest, fallocate),
		Op::FUSE_FLUSH => do_dispatch!(FlushRequest, flush),
		Op::FUSE_FORGET | Op::FUSE_BATCH_FORGET => {
			match ForgetRequest::from_fuse_request(&request) {
				Ok(request) => handlers.forget(ctx, &request),
				Err(err) => {
					if let Some(ref hooks) = hooks {
						hooks.request_error(header, err);
					}
				},
			};
			Ok(())
		},
		Op::FUSE_FSYNC => do_dispatch!(FsyncRequest, fsync),
		Op::FUSE_FSYNCDIR => do_dispatch!(FsyncdirRequest, fsyncdir),
		Op::FUSE_GETATTR => do_dispatch!(GetattrRequest, getattr),
		Op::FUSE_GETLK => do_dispatch!(GetlkRequest, getlk),
		Op::FUSE_GETXATTR => do_dispatch!(GetxattrRequest, getxattr),
		#[cfg(feature = "unstable_ioctl")]
		Op::FUSE_IOCTL => do_dispatch!(IoctlRequest, ioctl),
		Op::FUSE_LINK => do_dispatch!(LinkRequest, link),
		Op::FUSE_LISTXATTR => do_dispatch!(ListxattrRequest, listxattr),
		Op::FUSE_LOOKUP => do_dispatch!(LookupRequest, lookup),
		Op::FUSE_LSEEK => do_dispatch!(LseekRequest, lseek),
		Op::FUSE_MKDIR => do_dispatch!(MkdirRequest, mkdir),
		Op::FUSE_MKNOD => do_dispatch!(MknodRequest, mknod),
		Op::FUSE_OPEN => do_dispatch!(OpenRequest, open),
		Op::FUSE_OPENDIR => do_dispatch!(OpendirRequest, opendir),
		Op::FUSE_READ => do_dispatch!(ReadRequest, read),
		Op::FUSE_READDIR => do_dispatch!(ReaddirRequest, readdir),
		Op::FUSE_READLINK => do_dispatch!(ReadlinkRequest, readlink),
		Op::FUSE_RELEASE => do_dispatch!(ReleaseRequest, release),
		Op::FUSE_RELEASEDIR => do_dispatch!(ReleasedirRequest, releasedir),
		Op::FUSE_REMOVEXATTR => do_dispatch!(RemovexattrRequest, removexattr),
		Op::FUSE_RENAME | Op::FUSE_RENAME2 => {
			do_dispatch!(RenameRequest, rename)
		},
		Op::FUSE_RMDIR => do_dispatch!(RmdirRequest, rmdir),
		#[cfg(feature = "unstable_setattr")]
		Op::FUSE_SETATTR => do_dispatch!(SetattrRequest, setattr),
		Op::FUSE_SETLK | Op::FUSE_SETLKW => do_dispatch!(SetlkRequest, setlk),
		Op::FUSE_SETXATTR => do_dispatch!(SetxattrRequest, setxattr),
		Op::FUSE_STATFS => do_dispatch!(StatfsRequest, statfs),
		Op::FUSE_SYMLINK => do_dispatch!(SymlinkRequest, symlink),
		Op::FUSE_UNLINK => do_dispatch!(UnlinkRequest, unlink),
		Op::FUSE_WRITE => do_dispatch!(WriteRequest, write),
		_ => {
			if let Some(hooks) = hooks {
				let req = server::UnknownRequest::from_fuse_request(&request);
				hooks.unknown_request(&req);
			}
			let response = ErrorResponse::new(crate::Error::UNIMPLEMENTED);
			response.send(socket, &response_ctx)
		},
	}
}

/// User-provided handlers for FUSE operations.
///
/// Most FUSE handlers (with the exception of [`fuse_init`]) are asynchronous.
/// These handlers receive a [`ServerContext`] containing information about
/// the request itself, along with a [`ServerResponseWriter`] that must be used
/// to send the response.
///
/// The default implementation for all async handlers is to respond with
/// [`Error::UNIMPLEMENTED`].
///
/// [`fuse_init`]: #method.fuse_init
/// [`ServerContext`]: struct.ServerContext.html
/// [`ServerResponseWriter`]: struct.ServerResponseWriter.html
/// [`Error::UNIMPLEMENTED`]: crate::Error::UNIMPLEMENTED
#[allow(unused_variables)]
pub trait FuseHandlers<S: io::ServerSocket> {
	fn access(
		&self,
		ctx: ServerContext,
		request: &protocol::AccessRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::AccessResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_bmap"))]
	fn bmap(
		&self,
		ctx: ServerContext,
		request: &protocol::BmapRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::BmapResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn create(
		&self,
		ctx: ServerContext,
		request: &protocol::CreateRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::CreateResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn fallocate(
		&self,
		ctx: ServerContext,
		request: &protocol::FallocateRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FallocateResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn flush(
		&self,
		ctx: ServerContext,
		request: &protocol::FlushRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FlushResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn forget(
		&self,
		ctx: ServerContext,
		request: &protocol::ForgetRequest,
	) {
		if let Some(hooks) = ctx.hooks {
			hooks.unhandled_request(ctx.header);
		}
	}

	fn fsync(
		&self,
		ctx: ServerContext,
		request: &protocol::FsyncRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FsyncResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn fsyncdir(
		&self,
		ctx: ServerContext,
		request: &protocol::FsyncdirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FsyncdirResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn getattr(
		&self,
		ctx: ServerContext,
		request: &protocol::GetattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::GetattrResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn getlk(
		&self,
		ctx: ServerContext,
		request: &protocol::GetlkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::GetlkResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn getxattr(
		&self,
		ctx: ServerContext,
		request: &protocol::GetxattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::GetxattrResponse>, io::ServerSendError<S::Error>> {
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

	fn link(
		&self,
		ctx: ServerContext,
		request: &protocol::LinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::LinkResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn listxattr(
		&self,
		ctx: ServerContext,
		request: &protocol::ListxattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ListxattrResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn lookup(
		&self,
		ctx: ServerContext,
		request: &protocol::LookupRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::LookupResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn lseek(
		&self,
		ctx: ServerContext,
		request: &protocol::LseekRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::LseekResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn mkdir(
		&self,
		ctx: ServerContext,
		request: &protocol::MkdirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::MkdirResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn mknod(
		&self,
		ctx: ServerContext,
		request: &protocol::MknodRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::MknodResponse>, io::ServerSendError<S::Error>> {
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

	fn opendir(
		&self,
		ctx: ServerContext,
		request: &protocol::OpendirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::OpendirResponse>, io::ServerSendError<S::Error>> {
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

	fn readdir(
		&self,
		ctx: ServerContext,
		request: &protocol::ReaddirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReaddirResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn readlink(
		&self,
		ctx: ServerContext,
		request: &protocol::ReadlinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReadlinkResponse>, io::ServerSendError<S::Error>> {
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

	fn releasedir(
		&self,
		ctx: ServerContext,
		request: &protocol::ReleasedirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReleasedirResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn removexattr(
		&self,
		ctx: ServerContext,
		request: &protocol::RemovexattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::RemovexattrResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn rename(
		&self,
		ctx: ServerContext,
		request: &protocol::RenameRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::RenameResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn rmdir(
		&self,
		ctx: ServerContext,
		request: &protocol::RmdirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::RmdirResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_setattr"))]
	fn setattr(
		&self,
		ctx: ServerContext,
		request: &protocol::SetattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SetattrResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn setlk(
		&self,
		ctx: ServerContext,
		request: &protocol::SetlkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SetlkResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn setxattr(
		&self,
		ctx: ServerContext,
		request: &protocol::SetxattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SetxattrResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn statfs(
		&self,
		ctx: ServerContext,
		request: &protocol::StatfsRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::StatfsResponse>, io::ServerSendError<S::Error>> {
		#[cfg(not(target_os = "freebsd"))]
		{
			unhandled_request(ctx, send_reply)
		}

		#[cfg(target_os = "freebsd")]
		{
			if let Some(hooks) = ctx.hooks {
				hooks.unhandled_request(ctx.header);
			}
			let resp = protocol::StatfsResponse::new();
			send_reply.ok(&resp)
		}

	}

	fn symlink(
		&self,
		ctx: ServerContext,
		request: &protocol::SymlinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SymlinkResponse>, io::ServerSendError<S::Error>> {
		unhandled_request(ctx, send_reply)
	}

	fn unlink(
		&self,
		ctx: ServerContext,
		request: &protocol::UnlinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::UnlinkResponse>, io::ServerSendError<S::Error>> {
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
