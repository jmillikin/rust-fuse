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
use crate::server::{FuseConnection, FuseRequest, Reply};
use crate::server::basic::{
	NoopServerHooks,
	SendReply,
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
	pub fn serve(&self, buf: &mut impl io::Buffer) -> Result<(), io::Error<E>> {
		while let Some(request) = self.conn.recv(buf)? {
			fuse_request_dispatch(
				&self.conn,
				&self.handlers,
				self.hooks.as_ref(),
				request,
			)?;
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

pub(super) struct FuseReplySender<'a, S> {
	pub(super) conn: &'a FuseConnection<S>,
	pub(super) request_id: u64,
}

impl<S: OutputStream, R: Reply> SendReply<S, R> for FuseReplySender<'_, S> {
	fn ok(self, reply: &R) -> Result<(), Error<S::Error>> {
		self.conn.reply_ok(self.request_id, reply)
	}

	fn err(self, err: impl Into<NonZeroU16>) -> Result<(), Error<S::Error>> {
		self.conn.reply_err(self.request_id, err.into())
	}
}

fn fuse_request_dispatch<S: OutputStream>(
	conn: &FuseConnection<S>,
	handlers: &impl FuseHandlers<S>,
	hooks: Option<&impl ServerHooks>,
	request: FuseRequest,
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

	macro_rules! do_dispatch {
		($handler:tt) => {{
			match request.decode() {
				Ok(request) => {
					let reply_sender = FuseReplySender { conn, request_id };
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

	use crate::server::FuseOperation as FuseOp;
	match request.operation() {
		Some(FuseOp::Access) => do_dispatch!(access),
		#[cfg(feature = "unstable_bmap")]
		Some(FuseOp::Bmap) => do_dispatch!(bmap),
		Some(FuseOp::Create) => do_dispatch!(create),
		Some(FuseOp::Fallocate) => do_dispatch!(fallocate),
		Some(FuseOp::Flush) => do_dispatch!(flush),
		Some(FuseOp::Forget) => {
			let request = request.decode()?;
			handlers.forget(ctx, &request);
			Ok(())
		},
		Some(FuseOp::Fsync) => do_dispatch!(fsync),
		Some(FuseOp::Fsyncdir) => do_dispatch!(fsyncdir),
		Some(FuseOp::Getattr) => do_dispatch!(getattr),
		Some(FuseOp::Getlk) => do_dispatch!(getlk),
		Some(FuseOp::Getxattr) => do_dispatch!(getxattr),
		#[cfg(feature = "unstable_ioctl")]
		Some(FuseOp::Ioctl) => do_dispatch!(ioctl),
		Some(FuseOp::Link) => do_dispatch!(link),
		Some(FuseOp::Listxattr) => do_dispatch!(listxattr),
		Some(FuseOp::Lookup) => do_dispatch!(lookup),
		Some(FuseOp::Lseek) => do_dispatch!(lseek),
		Some(FuseOp::Mkdir) => do_dispatch!(mkdir),
		Some(FuseOp::Mknod) => do_dispatch!(mknod),
		Some(FuseOp::Open) => do_dispatch!(open),
		Some(FuseOp::Opendir) => do_dispatch!(opendir),
		Some(FuseOp::Read) => do_dispatch!(read),
		Some(FuseOp::Readdir) => do_dispatch!(readdir),
		Some(FuseOp::Readlink) => do_dispatch!(readlink),
		Some(FuseOp::Release) => do_dispatch!(release),
		Some(FuseOp::Releasedir) => do_dispatch!(releasedir),
		Some(FuseOp::Removexattr) => do_dispatch!(removexattr),
		Some(FuseOp::Rename) => do_dispatch!(rename),
		Some(FuseOp::Rmdir) => do_dispatch!(rmdir),
		#[cfg(feature = "unstable_setattr")]
		Some(FuseOp::Setattr) => do_dispatch!(setattr),
		Some(FuseOp::Setlk) => do_dispatch!(setlk),
		Some(FuseOp::Setxattr) => do_dispatch!(setxattr),
		Some(FuseOp::Statfs) => do_dispatch!(statfs),
		Some(FuseOp::Symlink) => do_dispatch!(symlink),
		Some(FuseOp::Unlink) => do_dispatch!(unlink),
		Some(FuseOp::Write) => do_dispatch!(write),
		_ => {
			if let Some(hooks) = hooks {
				let request = request.decode()?;
				hooks.unknown_request(&request);
			}
			conn.reply_err(request_id, ErrorCode::ENOSYS.into())
		},
	}
}