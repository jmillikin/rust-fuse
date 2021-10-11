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

use core::{cmp, fmt};
use core::num::NonZeroUsize;

#[cfg(feature = "respond_async")]
use std::sync::Arc;

use crate::channel::{self, WrapChannel};
use crate::cuse_handlers::CuseHandlers;
use crate::error::{Error, ErrorCode};
use crate::internal::fuse_kernel;
use crate::io::decode::{DecodeRequest, RequestBuf};
use crate::io::encode;
use crate::io::{self, Buffer, ProtocolVersion};
use crate::old_server as server;
use crate::protocol::common::DebugBytesAsString;
use crate::protocol::{CuseInitRequest, CuseInitResponse};
use crate::server::CuseRequest;

// CuseDeviceName {{{

#[derive(Hash)]
#[repr(transparent)]
pub struct CuseDeviceName([u8]);

impl CuseDeviceName {
	pub fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a CuseDeviceName> {
		if bytes.len() == 0 || bytes.contains(&0) {
			return None;
		}
		Some(unsafe { &*(bytes as *const [u8] as *const CuseDeviceName) })
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl fmt::Debug for CuseDeviceName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for CuseDeviceName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use core::fmt::Debug;
		DebugBytesAsString(&self.0).fmt(fmt)
	}
}

impl Eq for CuseDeviceName {}

impl PartialEq for CuseDeviceName {
	fn eq(&self, other: &Self) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for CuseDeviceName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl Ord for CuseDeviceName {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.as_bytes().cmp(&other.as_bytes())
	}
}

impl PartialEq<CuseDeviceName> for [u8] {
	fn eq(&self, other: &CuseDeviceName) -> bool {
		self.eq(other.as_bytes())
	}
}

impl PartialOrd for CuseDeviceName {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.as_bytes().partial_cmp(&other.as_bytes())
	}
}

// }}}

// CuseServerBuilder {{{

pub trait CuseServerChannel: server::ServerChannel {}

pub struct CuseServerBuilder<'a, Channel, Handlers, Hooks> {
	device_name: &'a CuseDeviceName,
	channel: Channel,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<'a, C, Handlers, Hooks> CuseServerBuilder<'a, C, Handlers, Hooks>
where
	C: CuseServerChannel,
	Handlers: CuseHandlers,
	Hooks: server::ServerHooks,
{
	pub fn new(
		device_name: &'a CuseDeviceName,
		channel: C,
		handlers: Handlers,
	) -> CuseServerBuilder<'a, C, Handlers, Hooks> {
		Self {
			device_name,
			channel,
			handlers,
			hooks: None,
		}
	}

	pub fn set_hooks(mut self, hooks: Hooks) -> Self {
		self.hooks = Some(hooks);
		self
	}

	pub fn build(mut self) -> Result<CuseServer<C, Handlers, Hooks>, C::Error> {
		let init_response = self.cuse_handshake()?;
		CuseServer::new(self.channel, self.handlers, self.hooks, &init_response)
	}

	fn cuse_handshake(&mut self) -> Result<CuseInitResponse, C::Error> {
		let mut read_buf = io::ArrayBuffer::new();
		let v_minor = ProtocolVersion::LATEST.minor();

		loop {
			let recv_len = self.channel.receive(read_buf.borrow_mut())?;
			let recv_len = NonZeroUsize::new(recv_len).unwrap(); // TODO
			let request_buf = match RequestBuf::new(&read_buf, recv_len) {
				Err(err) => {
					let err: Error = err.into();
					return Err(err.into());
				},
				Ok(x) => x,
			};

			let header = request_buf.header();
			if header.opcode != fuse_kernel::CUSE_INIT {
				return Err(Error::expected_cuse_init(header.opcode.0).into());
			}

			let request_id = header.unique;
			let init_request = CuseInitRequest::decode(request_buf, v_minor)
				.map_err(Error::from)?;
			let stream = WrapChannel(&self.channel);

			let version =
				match server::negotiate_version(init_request.version()) {
					Some(x) => x,
					None => {
						let mut init_response = CuseInitResponse::new();
						init_response.set_version(ProtocolVersion::LATEST);
						init_response.encode(
							encode::SyncSendOnce::new(&stream),
							request_id,
							None,
						)?;
						continue;
					},
				};

			#[allow(unused_mut)]
			let mut init_response = self.handlers.cuse_init(&init_request);
			init_response.set_version(version);

			#[cfg(not(feature = "std"))]
			init_response.set_max_write(cmp::min(
				init_response.max_write(),
				server::capped_max_write(),
			));

			init_response.encode(
				encode::SyncSendOnce::new(&stream),
				request_id,
				Some(self.device_name.as_bytes()),
			)?;
			return Ok(init_response);
		}
	}
}

// }}}

// CuseServer {{{

#[cfg(feature = "respond_async")]
pub struct CuseServer<Channel, Handlers, Hooks> {
	executor: CuseServerExecutor<Channel, Handlers, Hooks>,

	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	hooks: Option<Arc<Hooks>>,
	version: ProtocolVersion,
	read_buf_size: usize,
}

#[cfg(not(feature = "respond_async"))]
pub struct CuseServer<Channel, Handlers, Hooks> {
	executor: CuseServerExecutor<Channel, Handlers, Hooks>,
}

impl<C, Handlers, Hooks> CuseServer<C, Handlers, Hooks>
where
	C: CuseServerChannel,
	Handlers: CuseHandlers,
	Hooks: server::ServerHooks,
{
	#[cfg(feature = "respond_async")]
	fn new(
		channel: C,
		handlers: Handlers,
		hooks: Option<Hooks>,
		init_response: &CuseInitResponse,
	) -> Result<CuseServer<C, Handlers, Hooks>, C::Error> {
		let channel = Arc::new(channel);
		let handlers = Arc::new(handlers);
		let hooks = hooks.map(|h| Arc::new(h));
		let version = init_response.version();
		let read_buf_size = server::read_buf_size(init_response.max_write());

		let executor = CuseServerExecutor {
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
		init_response: &CuseInitResponse,
	) -> Result<CuseServer<C, Handlers, Hooks>, C::Error> {
		#[cfg(feature = "std")]
		let read_buf_size = server::read_buf_size(init_response.max_write());
		Ok(Self {
			executor: CuseServerExecutor {
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
	) -> &mut CuseServerExecutor<C, Handlers, Hooks> {
		&mut self.executor
	}

	#[cfg(feature = "respond_async")]
	#[cfg_attr(doc, doc(cfg(feature = "respond_async")))]
	pub fn new_executor(
		&self,
	) -> Result<CuseServerExecutor<C, Handlers, Hooks>, C::Error> {
		let channel = self.channel.as_ref().try_clone()?;
		Ok(CuseServerExecutor {
			channel: Arc::new(channel),
			handlers: self.handlers.clone(),
			hooks: self.hooks.as_ref().map(|h| h.clone()),
			version: self.version,
			read_buf_size: self.read_buf_size,
		})
	}
}

// }}}

// CuseServerExecutor {{{

#[cfg(feature = "respond_async")]
pub struct CuseServerExecutor<Channel, Handlers, Hooks> {
	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	hooks: Option<Arc<Hooks>>,
	version: ProtocolVersion,
	read_buf_size: usize,
}

#[cfg(not(feature = "respond_async"))]
pub struct CuseServerExecutor<Channel, Handlers, Hooks> {
	channel: Channel,
	handlers: Handlers,
	hooks: Option<Hooks>,
	version: ProtocolVersion,
	#[cfg(feature = "std")]
	read_buf_size: usize,
}

impl<C, Handlers, Hooks> CuseServerExecutor<C, Handlers, Hooks>
where
	C: CuseServerChannel,
	Handlers: CuseHandlers,
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
		server::main_loop(channel, &mut buf, true, |buf, recv_len| {
			let request = match CuseRequest::new(buf, recv_len, v_minor) {
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
			cuse_request_dispatch::<C, Handlers, Hooks>(
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
		Hooks: Send + Sync + 'static,
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
		server::main_loop(channel, &mut buf, true, |buf, recv_len| {
			let request = match CuseRequest::new(buf, recv_len, v_minor) {
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
			cuse_request_dispatch::<C, Handlers, Hooks>(
				request, handlers, respond, hooks,
			)?;
			channel_error
		})
	}
}

// }}}

fn cuse_request_dispatch<C, Handlers, Hooks>(
	request: CuseRequest,
	handlers: &Handlers,
	respond: server::RespondRef<C::T, Hooks::T>,
	#[cfg(feature = "respond_async")] hooks: Option<&Arc<Hooks::T>>,
	#[cfg(not(feature = "respond_async"))] hooks: Option<&Hooks::T>,
) -> Result<(), <<C as server::MaybeSendChannel>::T as channel::Channel>::Error>
where
	C: server::MaybeSendChannel,
	Handlers: CuseHandlers,
	Hooks: server::MaybeSendHooks,
{
	use server::ServerHooks;

	let header = request.header();
	let ctx = server::ServerContext::new(*header);

	if let Some(hooks) = hooks {
		hooks.request(ctx.request_header());
	}

	let stream = WrapChannel(respond.channel());

	#[rustfmt::skip]
	macro_rules! do_dispatch {
		($handler:tt) => {{
			match request.decode() {
				Ok(request) => {
					handlers.$handler(ctx, &request, respond);
					Ok(())
				},
				Err(err) => {
					if let Some(hooks) = hooks {
						hooks.request_error(header, err.into());
					}
					encode::ReplyEncoder::new(
						encode::SyncSendOnce::new(&stream),
						header.request_id(),
					).encode_error(ErrorCode::EIO.into())
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
			if let Some(hooks) = hooks {
				let request = request.decode().map_err(Error::from)?;
				hooks.unknown_request(&request);
			}
			encode::ReplyEncoder::new(
				encode::SyncSendOnce::new(&stream),
				header.request_id(),
			).encode_error(ErrorCode::ENOSYS.into())
		},
	}
}
