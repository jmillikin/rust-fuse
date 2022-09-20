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
use crate::server::{FuseConnection, FuseRequest, Reply, ServerError};
use crate::server::basic::{
	NoopServerHooks,
	SendReply,
	SentReply,
	ServerContext,
	ServerHooks,
};
use crate::server::basic::fuse_handlers::FuseHandlers;

pub struct FuseServer<S, Handlers, Hooks> {
	conn: FuseConnection<S>,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<S, Handlers, Hooks> FuseServer<S, Handlers, Hooks>
where
	S: io::FuseServerSocket,
	Handlers: FuseHandlers<S>,
	Hooks: ServerHooks,
{
	pub fn serve(&self, buf: &mut [u8]) -> Result<(), ServerError<S::Error>> {
		while let Some(request) = self.conn.recv(buf)? {
			let result = fuse_request_dispatch(
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

pub struct FuseServerBuilder<S, Handlers, Hooks> {
	conn: FuseConnection<S>,
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

impl<S: io::FuseServerSocket> SendReply<S> for FuseReplySender<'_, S> {
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

fn fuse_request_dispatch<S: io::FuseServerSocket>(
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
				let request = request.into_unknown();
				hooks.unknown_request(&request);
			}
			conn.reply_err(request_id, crate::Error::UNIMPLEMENTED)
		},
	}
}
