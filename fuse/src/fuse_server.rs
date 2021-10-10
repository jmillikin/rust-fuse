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

#[cfg(not(feature = "std"))]
use core::cmp;

#[cfg(feature = "respond_async")]
use std::sync::Arc;

use crate::channel::{self, WrapChannel};
use crate::error::{Error, ErrorCode};
use crate::fuse_handlers::FuseHandlers;
use crate::internal::fuse_kernel;
use crate::io::{self, Buffer, ProtocolVersion};
use crate::io::encode::{self, EncodeReply};
use crate::old_server as server;
use crate::protocol::{FuseInitRequest, FuseInitResponse};
use crate::server::FuseRequest;

// FuseServerBuilder {{{

pub trait FuseServerChannel: server::ServerChannel {}

pub struct FuseServerBuilder<Channel, Handlers, Hooks> {
	channel: Channel,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<C, Handlers, Hooks> FuseServerBuilder<C, Handlers, Hooks>
where
	C: FuseServerChannel,
	Handlers: FuseHandlers,
	Hooks: server::ServerHooks,
{
	pub fn new(
		channel: C,
		handlers: Handlers,
	) -> FuseServerBuilder<C, Handlers, Hooks> {
		Self {
			channel,
			handlers,
			hooks: None,
		}
	}

	pub fn set_hooks(mut self, hooks: Hooks) -> Self {
		self.hooks = Some(hooks);
		self
	}

	pub fn build(mut self) -> Result<FuseServer<C, Handlers, Hooks>, C::Error> {
		let init_response = self.fuse_handshake()?;
		FuseServer::new(self.channel, self.handlers, self.hooks, &init_response)
	}

	fn fuse_handshake(&mut self) -> Result<FuseInitResponse, C::Error> {
		let mut read_buf = io::ArrayBuffer::new();
		let v_minor = ProtocolVersion::LATEST.minor();

		loop {
			let recv_len = self.channel.receive(read_buf.borrow_mut())?;
			let request = match FuseRequest::new(&read_buf, recv_len, v_minor) {
				Err(err) => {
					let err: Error = err.into();
					return Err(err.into());
				},
				Ok(x) => x,
			};

			if request.opcode() != fuse_kernel::FUSE_INIT {
				return Err(
					Error::expected_fuse_init(request.opcode().0).into()
				);
			}

			let request_id = request.header().request_id();
			let init_request: FuseInitRequest =
				request.decode().map_err(Error::from)?;
			let stream = WrapChannel(&self.channel);

			let version =
				match server::negotiate_version(init_request.version()) {
					Some(x) => x,
					None => {
						let mut init_response = FuseInitResponse::new();
						init_response.set_version(ProtocolVersion::LATEST);
						init_response.encode(
							encode::SyncSendOnce::new(&stream),
							request_id,
							// FuseInitResponse always encodes with its own version
							ProtocolVersion::LATEST.minor(),
						)?;
						continue;
					},
				};

			#[allow(unused_mut)]
			let mut init_response = self.handlers.fuse_init(&init_request);
			init_response.set_version(version);

			#[cfg(not(feature = "std"))]
			init_response.set_max_write(cmp::min(
				init_response.max_write(),
				server::capped_max_write(),
			));

			init_response.encode(
				encode::SyncSendOnce::new(&stream),
				request_id,
				// FuseInitResponse always encodes with its own version
				ProtocolVersion::LATEST.minor(),
			)?;
			return Ok(init_response);
		}
	}
}

// }}}

// FuseServer {{{

#[cfg(feature = "respond_async")]
pub struct FuseServer<Channel, Handlers, Hooks> {
	executor: FuseServerExecutor<Channel, Handlers, Hooks>,

	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	hooks: Option<Arc<Hooks>>,
	version: ProtocolVersion,
	read_buf_size: usize,
}

#[cfg(not(feature = "respond_async"))]
pub struct FuseServer<Channel, Handlers, Hooks> {
	executor: FuseServerExecutor<Channel, Handlers, Hooks>,
}

impl<C, Handlers, Hooks> FuseServer<C, Handlers, Hooks>
where
	C: FuseServerChannel,
	Handlers: FuseHandlers,
	Hooks: server::ServerHooks,
{
	#[cfg(feature = "respond_async")]
	fn new(
		channel: C,
		handlers: Handlers,
		hooks: Option<Hooks>,
		init_response: &FuseInitResponse,
	) -> Result<FuseServer<C, Handlers, Hooks>, C::Error> {
		let channel = Arc::new(channel);
		let handlers = Arc::new(handlers);
		let hooks = hooks.map(|h| Arc::new(h));
		let version = init_response.version();
		let read_buf_size = server::read_buf_size(init_response.max_write());

		let executor = FuseServerExecutor {
			channel: channel.clone(),
			handlers: handlers.clone(),
			hooks: hooks.clone(),
			version,
			read_buf_size,
		};

		Ok(Self {
			executor,
			channel,
			handlers,
			hooks,
			version,
			read_buf_size,
		})
	}

	#[cfg(not(feature = "respond_async"))]
	fn new(
		channel: C,
		handlers: Handlers,
		hooks: Option<Hooks>,
		init_response: &FuseInitResponse,
	) -> Result<FuseServer<C, Handlers, Hooks>, C::Error> {
		#[cfg(feature = "std")]
		let read_buf_size = server::read_buf_size(init_response.max_write());
		Ok(Self {
			executor: FuseServerExecutor {
				channel,
				handlers,
				hooks,
				version: init_response.version(),
				#[cfg(feature = "std")]
				read_buf_size,
			},
		})
	}

	pub fn executor_mut(
		&mut self,
	) -> &mut FuseServerExecutor<C, Handlers, Hooks> {
		&mut self.executor
	}

	#[cfg(feature = "respond_async")]
	#[cfg_attr(doc, doc(cfg(feature = "respond_async")))]
	pub fn new_executor(
		&self,
	) -> Result<FuseServerExecutor<C, Handlers, Hooks>, C::Error> {
		let channel = self.channel.as_ref().try_clone()?;
		Ok(FuseServerExecutor {
			channel: Arc::new(channel),
			handlers: self.handlers.clone(),
			hooks: self.hooks.as_ref().map(|h| h.clone()),
			version: self.version,
			read_buf_size: self.read_buf_size,
		})
	}
}

// }}}

// FuseServerExecutor {{{

#[cfg(feature = "respond_async")]
pub struct FuseServerExecutor<Channel, Handlers, Hooks> {
	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	hooks: Option<Arc<Hooks>>,
	version: ProtocolVersion,
	read_buf_size: usize,
}

#[cfg(not(feature = "respond_async"))]
pub struct FuseServerExecutor<Channel, Handlers, Hooks> {
	channel: Channel,
	handlers: Handlers,
	hooks: Option<Hooks>,
	version: ProtocolVersion,
	#[cfg(feature = "std")]
	read_buf_size: usize,
}

impl<C, Handlers, Hooks> FuseServerExecutor<C, Handlers, Hooks>
where
	C: FuseServerChannel,
	Handlers: FuseHandlers,
	Hooks: server::ServerHooks,
{
	#[cfg(feature = "respond_async")]
	pub fn run(&mut self) -> Result<(), C::Error>
	where
		C: Send + Sync + 'static,
		Hooks: Send + Sync + 'static,
	{
		let channel = self.channel.as_ref();
		let handlers = self.handlers.as_ref();
		let hooks = self.hooks.as_deref();
		let v_minor = self.version.minor();
		let mut buf = io::PinnedBuffer::new(self.read_buf_size);
		server::main_loop(channel, &mut buf, false, |buf, recv_len| {
			let request = match FuseRequest::new(buf, recv_len, v_minor) {
				Err(err) => {
					let err: Error = err.into();
					return Err(err.into());
				},
				Ok(x) => x,
			};
			let mut channel_err = Ok(());
			let respond = server::RespondRef::new(
				channel,
				hooks,
				&mut channel_err,
				request.header(),
				self.version,
				&self.channel,
				self.hooks.as_ref(),
			);
			fuse_request_dispatch::<C, Handlers, Hooks>(
				request,
				handlers,
				respond,
				self.hooks.as_ref(),
			)?;
			channel_err
		})
	}

	#[cfg(not(feature = "respond_async"))]
	pub fn run(&mut self) -> Result<(), C::Error>
	where
		C: Send + Sync + 'static,
	{
		self.run_local()
	}

	#[cfg(any(doc, not(feature = "respond_async")))]
	#[cfg_attr(doc, doc(cfg(not(feature = "respond_async"))))]
	pub fn run_local(&mut self) -> Result<(), C::Error> {
		let channel = &self.channel;
		let handlers = &self.handlers;
		let hooks = self.hooks.as_ref();
		let v_minor = self.version.minor();
		#[cfg(feature = "std")]
		let mut buf = io::PinnedBuffer::new(self.read_buf_size);
		#[cfg(not(feature = "std"))]
		let mut buf = io::ArrayBuffer::new();
		server::main_loop(channel, &mut buf, false, |buf, recv_len| {
			let request = match FuseRequest::new(buf, recv_len, v_minor) {
				Err(err) => {
					let err: Error = err.into();
					return Err(err.into());
				},
				Ok(x) => x,
			};
			let mut channel_error = Ok(());
			let respond = server::RespondRef::new(
				channel,
				hooks,
				&mut channel_error,
				request.header(),
				self.version,
			);
			fuse_request_dispatch::<C, Handlers, Hooks>(
				request, handlers, respond, hooks,
			)?;
			channel_error
		})
	}
}

// }}}

fn fuse_request_dispatch<C, Handlers, Hooks>(
	request: FuseRequest,
	handlers: &Handlers,
	respond: server::RespondRef<C::T, Hooks::T>,
	#[cfg(feature = "respond_async")] hooks: Option<&Arc<Hooks::T>>,
	#[cfg(not(feature = "respond_async"))] hooks: Option<&Hooks::T>,
) -> Result<(), <<C as server::MaybeSendChannel>::T as channel::Channel>::Error>
where
	C: server::MaybeSendChannel,
	Handlers: FuseHandlers,
	Hooks: server::MaybeSendHooks,
{
	use crate::old_server::ServerHooks;

	let header = request.header();
	let ctx = server::ServerContext::new(*header);

	if let Some(hooks) = hooks {
		hooks.request(ctx.request_header());
	}

	let stream = WrapChannel(respond.channel());

	macro_rules! do_dispatch {
		($handler:tt) => {{
			match request.decode() {
				Ok(request) => handlers.$handler(ctx, &request, respond),
				Err(err) => {
					if let Some(hooks) = hooks {
						hooks.request_error(header, err.into())
					}
					encode::ReplyEncoder::new(
						encode::SyncSendOnce::new(&stream),
						header.request_id(),
					).encode_error(ErrorCode::EIO.into())?;
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
			let request = request.decode().map_err(Error::from)?;
			handlers.forget(ctx, &request);
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
				let request = request.decode().map_err(Error::from)?;
				hooks.unknown_request(&request);
			}
			encode::ReplyEncoder::new(
				encode::SyncSendOnce::new(&stream),
				header.request_id(),
			).encode_error(ErrorCode::ENOSYS.into())?;
		},
	}
	Ok(())
}
