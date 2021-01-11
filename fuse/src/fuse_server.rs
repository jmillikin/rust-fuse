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

use crate::channel;
use crate::error::{Error, ErrorCode};
use crate::fuse_handlers::FuseHandlers;
use crate::internal::fuse_io::{
	self,
	AlignedBuffer,
	DecodeRequest,
	EncodeResponse,
};
use crate::internal::fuse_kernel;
use crate::internal::types::ProtocolVersion;
use crate::protocol::common::{RequestHeader, UnknownRequest};
use crate::protocol::{FuseInitRequest, FuseInitResponse};
use crate::server;

const FUSE: fuse_io::Semantics = fuse_io::Semantics::FUSE;

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
		let mut read_buf = fuse_io::MinReadBuffer::new();

		loop {
			let request_size = self.channel.receive(read_buf.get_mut())?;
			let request_buf = fuse_io::aligned_slice(&read_buf, request_size);
			let request_decoder = fuse_io::RequestDecoder::new(
				request_buf,
				ProtocolVersion::LATEST,
				FUSE,
			)?;

			let request_header = request_decoder.header();
			if request_header.opcode != fuse_kernel::FUSE_INIT {
				return Err(
					Error::expected_fuse_init(request_header.opcode.0).into()
				);
			}

			let request_id = request_header.unique;
			let init_request =
				FuseInitRequest::decode_request(request_decoder)?;

			let encoder = fuse_io::ResponseEncoder::new(
				&self.channel,
				request_id,
				// FuseInitResponse always encodes with its own version
				ProtocolVersion::LATEST,
			);

			let version =
				match server::negotiate_version(init_request.version()) {
					Some(x) => x,
					None => {
						let mut init_response = FuseInitResponse::new();
						init_response.set_version(ProtocolVersion::LATEST);
						init_response.encode_response(encoder)?;
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

			init_response.encode_response(encoder)?;
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
		let mut buf = fuse_io::AlignedVec::new(self.read_buf_size);
		server::main_loop(channel, &mut buf, self.version, FUSE, |dec| {
			let mut channel_err = Ok(());
			let respond = server::RespondRef::new(
				channel,
				hooks,
				&mut channel_err,
				RequestHeader::new_ref(dec.header()),
				self.version,
				&self.channel,
				self.hooks.as_ref(),
			);
			fuse_request_dispatch::<C, Handlers, Hooks>(
				dec,
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
		#[cfg(feature = "std")]
		let mut buf = fuse_io::AlignedVec::new(self.read_buf_size);
		#[cfg(not(feature = "std"))]
		let mut buf = fuse_io::MinReadBuffer::new();
		server::main_loop(channel, &mut buf, self.version, FUSE, |dec| {
			let mut channel_error = Ok(());
			let respond = server::RespondRef::new(
				channel,
				hooks,
				&mut channel_error,
				RequestHeader::new_ref(dec.header()),
				self.version,
			);
			fuse_request_dispatch::<C, Handlers, Hooks>(
				dec, handlers, respond, hooks,
			)?;
			channel_error
		})
	}
}

// }}}

fn fuse_request_dispatch<C, Handlers, Hooks>(
	request_decoder: fuse_io::RequestDecoder,
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
	use crate::server::ServerHooks;

	let header = request_decoder.header();
	let ctx = server::ServerContext::new(*header);

	if let Some(hooks) = hooks {
		hooks.request(ctx.request_header());
	}

	macro_rules! do_dispatch {
		($handler:tt) => {{
			match DecodeRequest::decode_request(request_decoder) {
				Ok(request) => handlers.$handler(ctx, &request, respond),
				Err(err) => {
					if let Some(hooks) = hooks {
						hooks.request_error(RequestHeader::new_ref(header), err)
					}
					respond.encoder().encode_error(ErrorCode::EIO)?;
				},
			}
		}};
	}

	match header.opcode {
		#[cfg(feature = "unstable_access")]
		fuse_kernel::FUSE_ACCESS => do_dispatch!(access),
		#[cfg(feature = "unstable_bmap")]
		fuse_kernel::FUSE_BMAP => do_dispatch!(bmap),
		#[cfg(feature = "unstable_create")]
		fuse_kernel::FUSE_CREATE => do_dispatch!(create),
		#[cfg(feature = "unstable_fallocate")]
		fuse_kernel::FUSE_FALLOCATE => do_dispatch!(fallocate),
		#[cfg(feature = "unstable_flush")]
		fuse_kernel::FUSE_FLUSH => do_dispatch!(flush),
		fuse_kernel::FUSE_FORGET | fuse_kernel::FUSE_BATCH_FORGET => {
			let request = DecodeRequest::decode_request(request_decoder)?;
			handlers.forget(ctx, &request);
		},
		fuse_kernel::FUSE_FSYNC => do_dispatch!(fsync),
		fuse_kernel::FUSE_FSYNCDIR => do_dispatch!(fsyncdir),
		fuse_kernel::FUSE_GETATTR => do_dispatch!(getattr),
		#[cfg(feature = "unstable_getlk")]
		fuse_kernel::FUSE_GETLK => do_dispatch!(getlk),
		fuse_kernel::FUSE_GETXATTR => do_dispatch!(getxattr),
		#[cfg(feature = "unstable_ioctl")]
		fuse_kernel::FUSE_IOCTL => do_dispatch!(ioctl),
		fuse_kernel::FUSE_LINK => do_dispatch!(link),
		fuse_kernel::FUSE_LISTXATTR => do_dispatch!(listxattr),
		fuse_kernel::FUSE_LOOKUP => do_dispatch!(lookup),
		#[cfg(feature = "unstable_lseek")]
		fuse_kernel::FUSE_LSEEK => do_dispatch!(lseek),
		fuse_kernel::FUSE_MKDIR => do_dispatch!(mkdir),
		fuse_kernel::FUSE_MKNOD => do_dispatch!(mknod),
		fuse_kernel::FUSE_OPEN => do_dispatch!(open),
		fuse_kernel::FUSE_OPENDIR => do_dispatch!(opendir),
		fuse_kernel::FUSE_READ => do_dispatch!(read),
		fuse_kernel::FUSE_READDIR => do_dispatch!(readdir),
		fuse_kernel::FUSE_READLINK => do_dispatch!(readlink),
		fuse_kernel::FUSE_RELEASE => do_dispatch!(release),
		fuse_kernel::FUSE_RELEASEDIR => do_dispatch!(releasedir),
		fuse_kernel::FUSE_REMOVEXATTR => do_dispatch!(removexattr),
		fuse_kernel::FUSE_RENAME | fuse_kernel::FUSE_RENAME2 => {
			do_dispatch!(rename)
		},
		fuse_kernel::FUSE_RMDIR => do_dispatch!(rmdir),
		#[cfg(feature = "unstable_setattr")]
		fuse_kernel::FUSE_SETATTR => do_dispatch!(setattr),
		#[cfg(feature = "unstable_setlk")]
		fuse_kernel::FUSE_SETLK => do_dispatch!(setlk),
		fuse_kernel::FUSE_SETXATTR => do_dispatch!(setxattr),
		fuse_kernel::FUSE_STATFS => do_dispatch!(statfs),
		fuse_kernel::FUSE_SYMLINK => do_dispatch!(symlink),
		fuse_kernel::FUSE_UNLINK => do_dispatch!(unlink),
		fuse_kernel::FUSE_WRITE => do_dispatch!(write),
		_ => {
			if let Some(hooks) = hooks {
				let request = UnknownRequest::decode_request(request_decoder)?;
				hooks.unknown_request(&request);
			}
			respond.encoder().encode_error(ErrorCode::ENOSYS)?;
		},
	}
	Ok(())
}
