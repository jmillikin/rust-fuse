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

use crate::io::{self, OutputStream, SendError};
use crate::server::{FuseConnection, FuseRequest, Reply, ServerError};
use crate::server::basic::{
	NoopServerHooks,
	SendReply,
	SentReply,
	ServerContext,
	ServerHooks,
};
use crate::server::basic::fuse_handlers::FuseHandlers;

pub struct FuseServer<Stream, Handlers, Hooks> {
	conn: FuseConnection<Stream>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, E, Handlers, Hooks> FuseServer<S, Handlers, Hooks>
where
	S: io::InputStream<Error = E> + io::OutputStream<Error = E>,
	Handlers: FuseHandlers<S>,
	Hooks: ServerHooks,
{
	pub fn serve(&self, buf: &mut impl io::Buffer) -> Result<(), ServerError<E>> {
		while let Some(request) = self.conn.recv(buf)? {
			let result = fuse_request_dispatch(
				&self.conn,
				&self.handlers,
				self.hooks.as_ref(),
				request,
			);
			match result {
				Ok(()) => {},
				Err(SendError::NotFound) => {},
				Err(SendError::Other(err)) => return Err(ServerError::SendError(err)),
			};
		}
		Ok(())
	}
}

pub struct FuseServerBuilder<Stream, Handlers, Hooks> {
	conn: FuseConnection<Stream>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, Handlers> FuseServerBuilder<S, Handlers, NoopServerHooks> {
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
			conn: self.conn,
			handlers: self.handlers,
			hooks: self.hooks,
		}
	}
}

struct FuseReplySender<'a, S> {
	conn: &'a FuseConnection<S>,
	request_id: u64,
	sent_reply: &'a mut bool,
}

impl<S: OutputStream> SendReply<S> for FuseReplySender<'_, S> {
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

fn fuse_request_dispatch<S: OutputStream>(
	conn: &FuseConnection<S>,
	handlers: &impl FuseHandlers<S>,
	hooks: Option<&impl ServerHooks>,
	request: FuseRequest,
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
			match <$req_type>::from_fuse_request(&request) {
				Ok(request) => {
					let mut sent_reply = false;
					let reply_sender = FuseReplySender {
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

	use crate::server::FuseOperation as FuseOp;
	use crate::protocol::*;
	match request.operation() {
		Some(FuseOp::Access) => do_dispatch!(AccessRequest, access),
		#[cfg(feature = "unstable_bmap")]
		Some(FuseOp::Bmap) => do_dispatch!(BmapRequest, bmap),
		Some(FuseOp::Create) => do_dispatch!(CreateRequest, create),
		Some(FuseOp::Fallocate) => do_dispatch!(FallocateRequest, fallocate),
		Some(FuseOp::Flush) => do_dispatch!(FlushRequest, flush),
		Some(FuseOp::Forget) => {
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
		Some(FuseOp::Fsync) => do_dispatch!(FsyncRequest, fsync),
		Some(FuseOp::Fsyncdir) => do_dispatch!(FsyncdirRequest, fsyncdir),
		Some(FuseOp::Getattr) => do_dispatch!(GetattrRequest, getattr),
		Some(FuseOp::Getlk) => do_dispatch!(GetlkRequest, getlk),
		Some(FuseOp::Getxattr) => do_dispatch!(GetxattrRequest, getxattr),
		#[cfg(feature = "unstable_ioctl")]
		Some(FuseOp::Ioctl) => do_dispatch!(IoctlRequest, ioctl),
		Some(FuseOp::Link) => do_dispatch!(LinkRequest, link),
		Some(FuseOp::Listxattr) => do_dispatch!(ListxattrRequest, listxattr),
		Some(FuseOp::Lookup) => do_dispatch!(LookupRequest, lookup),
		Some(FuseOp::Lseek) => do_dispatch!(LseekRequest, lseek),
		Some(FuseOp::Mkdir) => do_dispatch!(MkdirRequest, mkdir),
		Some(FuseOp::Mknod) => do_dispatch!(MknodRequest, mknod),
		Some(FuseOp::Open) => do_dispatch!(OpenRequest, open),
		Some(FuseOp::Opendir) => do_dispatch!(OpendirRequest, opendir),
		Some(FuseOp::Read) => do_dispatch!(ReadRequest, read),
		Some(FuseOp::Readdir) => do_dispatch!(ReaddirRequest, readdir),
		Some(FuseOp::Readlink) => do_dispatch!(ReadlinkRequest, readlink),
		Some(FuseOp::Release) => do_dispatch!(ReleaseRequest, release),
		Some(FuseOp::Releasedir) => do_dispatch!(ReleasedirRequest, releasedir),
		Some(FuseOp::Removexattr) => do_dispatch!(RemovexattrRequest, removexattr),
		Some(FuseOp::Rename) => do_dispatch!(RenameRequest, rename),
		Some(FuseOp::Rmdir) => do_dispatch!(RmdirRequest, rmdir),
		#[cfg(feature = "unstable_setattr")]
		Some(FuseOp::Setattr) => do_dispatch!(SetattrRequest, setattr),
		Some(FuseOp::Setlk) => do_dispatch!(SetlkRequest, setlk),
		Some(FuseOp::Setxattr) => do_dispatch!(SetxattrRequest, setxattr),
		Some(FuseOp::Statfs) => do_dispatch!(StatfsRequest, statfs),
		Some(FuseOp::Symlink) => do_dispatch!(SymlinkRequest, symlink),
		Some(FuseOp::Unlink) => do_dispatch!(UnlinkRequest, unlink),
		Some(FuseOp::Write) => do_dispatch!(WriteRequest, write),
		_ => {
			if let Some(hooks) = hooks {
				let request = request.into_unknown();
				hooks.unknown_request(&request);
			}
			conn.reply_err(request_id, crate::Error::UNIMPLEMENTED)
		},
	}
}
