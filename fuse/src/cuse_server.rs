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

#[cfg(feature = "std")]
use std::sync::Arc;

use crate::channel;
use crate::cuse_handlers::CuseHandlers;
use crate::error::{Error, ErrorCode};
use crate::internal::fuse_io::{self, AlignedBuffer, DecodeRequest};
use crate::internal::fuse_kernel;
use crate::internal::types::ProtocolVersion;
use crate::protocol::common::DebugBytesAsString;
#[cfg(feature = "std")]
use crate::protocol::common::UnknownRequest;
use crate::protocol::{CuseInitRequest, CuseInitResponse};
use crate::server;

const CUSE: fuse_io::Semantics = fuse_io::Semantics::CUSE;

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

pub struct CuseServerBuilder<'a, Channel, Handlers> {
	device_name: &'a CuseDeviceName,
	channel: Channel,
	handlers: Handlers,
	#[cfg(feature = "std")]
	hooks: Option<Box<dyn server::ServerHooks>>,
}

impl<'a, C, H> CuseServerBuilder<'a, C, H>
where
	C: CuseServerChannel,
	H: CuseHandlers,
{
	pub fn new(
		device_name: &'a CuseDeviceName,
		channel: C,
		handlers: H,
	) -> CuseServerBuilder<'a, C, H> {
		Self {
			device_name,
			channel,
			handlers,
			#[cfg(feature = "std")]
			hooks: None,
		}
	}

	#[cfg(feature = "std")]
	#[cfg_attr(doc, doc(cfg(feature = "std")))]
	pub fn set_hooks(mut self, hooks: Box<dyn server::ServerHooks>) -> Self {
		self.hooks = Some(hooks);
		self
	}

	pub fn build(mut self) -> Result<CuseServer<C, H>, C::Error> {
		let init_response = self.cuse_handshake()?;
		CuseServer::new(
			self.channel,
			self.handlers,
			#[cfg(feature = "std")]
			self.hooks,
			&init_response,
		)
	}

	fn cuse_handshake(&mut self) -> Result<CuseInitResponse, C::Error> {
		let mut read_buf = fuse_io::MinReadBuffer::new();

		loop {
			let request_size = self.channel.receive(read_buf.get_mut())?;
			let request_buf = fuse_io::aligned_slice(&read_buf, request_size);
			let request_decoder = fuse_io::RequestDecoder::new(
				request_buf,
				ProtocolVersion::LATEST,
				CUSE,
			)?;

			let request_header = request_decoder.header();
			if request_header.opcode != fuse_kernel::CUSE_INIT {
				return Err(
					Error::expected_cuse_init(request_header.opcode.0).into()
				);
			}

			let request_id = request_header.unique;
			let init_request =
				CuseInitRequest::decode_request(request_decoder)?;

			let encoder = fuse_io::ResponseEncoder::new(
				&self.channel,
				request_id,
				// CuseInitResponse always encodes with its own version
				ProtocolVersion::LATEST,
			);

			let version =
				match server::negotiate_version(init_request.version()) {
					Some(x) => x,
					None => {
						let mut init_response = CuseInitResponse::new();
						init_response.set_version(ProtocolVersion::LATEST);
						init_response.encode_response(encoder, None)?;
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

			init_response
				.encode_response(encoder, Some(self.device_name.as_bytes()))?;
			return Ok(init_response);
		}
	}
}

// }}}

// CuseServer {{{

#[cfg(feature = "std")]
pub struct CuseServer<Channel, Handlers> {
	executor: CuseServerExecutor<Channel, Handlers>,

	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	hooks: Option<Arc<dyn server::ServerHooks>>,
	version: ProtocolVersion,
	read_buf_size: usize,
}

#[cfg(not(feature = "std"))]
pub struct CuseServer<Channel, Handlers> {
	executor: CuseServerExecutor<Channel, Handlers>,
}

impl<C, H> CuseServer<C, H>
where
	C: CuseServerChannel,
	H: CuseHandlers,
{
	#[cfg(feature = "std")]
	fn new(
		channel: C,
		handlers: H,
		hooks: Option<Box<dyn server::ServerHooks>>,
		init_response: &CuseInitResponse,
	) -> Result<CuseServer<C, H>, C::Error> {
		let channel = Arc::new(channel);
		let handlers = Arc::new(handlers);
		let hooks = hooks.map(|h| Arc::from(h));
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

	#[cfg(not(feature = "std"))]
	fn new(
		channel: C,
		handlers: H,
		init_response: &CuseInitResponse,
	) -> Result<CuseServer<C, H>, C::Error> {
		Ok(Self {
			executor: CuseServerExecutor {
				channel,
				handlers,
				version: init_response.version(),
			},
		})
	}

	pub fn executor_mut(&mut self) -> &mut CuseServerExecutor<C, H> {
		&mut self.executor
	}

	#[cfg(feature = "std")]
	#[cfg_attr(doc, doc(cfg(feature = "std")))]
	pub fn new_executor(&self) -> Result<CuseServerExecutor<C, H>, C::Error> {
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

#[cfg(feature = "std")]
pub struct CuseServerExecutor<Channel, Handlers> {
	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	hooks: Option<Arc<dyn server::ServerHooks>>,
	version: ProtocolVersion,
	read_buf_size: usize,
}

#[cfg(not(feature = "std"))]
pub struct CuseServerExecutor<Channel, Handlers> {
	channel: Channel,
	handlers: Handlers,
	version: ProtocolVersion,
}

impl<C, H> CuseServerExecutor<C, H>
where
	C: CuseServerChannel,
	H: CuseHandlers,
{
	#[cfg(feature = "std")]
	pub fn run(&mut self) -> Result<(), C::Error>
	where
		C: Send + Sync + 'static,
	{
		let channel = self.channel.as_ref();
		let handlers = self.handlers.as_ref();
		let mut buf = fuse_io::AlignedVec::new(self.read_buf_size);
		server::main_loop(channel, &mut buf, self.version, CUSE, |dec| {
			let request_id = dec.header().unique;
			let respond = server::RespondRef::new(
				channel,
				request_id,
				self.version,
				&self.channel,
			);
			cuse_request_dispatch::<C, H>(dec, handlers, respond, &self.hooks)
		})
	}

	#[cfg(not(feature = "std"))]
	pub fn run(&mut self) -> Result<(), C::Error>
	where
		C: Send + Sync + 'static,
	{
		self.run_local()
	}

	#[cfg(any(doc, not(feature = "std")))]
	#[cfg_attr(doc, doc(cfg(not(feature = "std"))))]
	pub fn run_local(&mut self) -> Result<(), C::Error> {
		let channel = &self.channel;
		let handlers = &self.handlers;
		let mut buf = fuse_io::MinReadBuffer::new();
		server::main_loop(channel, &mut buf, self.version, CUSE, |dec| {
			let request_id = dec.header().unique;
			let respond =
				server::RespondRef::new(channel, request_id, self.version);
			cuse_request_dispatch::<C, H>(dec, handlers, respond)
		})
	}
}

// }}}

fn cuse_request_dispatch<C, H>(
	request_decoder: fuse_io::RequestDecoder,
	handlers: &H,
	respond: server::RespondRef<C::T>,
	#[cfg(feature = "std")] hooks: &Option<Arc<dyn server::ServerHooks>>,
) -> Result<(), <<C as server::MaybeSendChannel>::T as channel::Channel>::Error>
where
	C: server::MaybeSendChannel,
	H: CuseHandlers,
{
	let header = request_decoder.header();
	let ctx = server::ServerContext::new(*header);

	#[cfg(feature = "std")]
	if let Some(hooks) = hooks {
		hooks.on_request(ctx.request_header());
	}

	macro_rules! do_dispatch {
		($handler:tt) => {{
			match DecodeRequest::decode_request(request_decoder) {
				Ok(request) => handlers.$handler(ctx, &request, respond),
				Err(err) => {
					// TODO: use ServerLogger to log the parse error
					let _ = err;
					respond.encoder().encode_error(ErrorCode::EIO)?;
				},
			}
		}};
	}

	match header.opcode {
		#[cfg(feature = "unstable_flush")]
		fuse_kernel::FUSE_FLUSH => do_dispatch!(flush),
		#[cfg(feature = "unstable_fsync")]
		fuse_kernel::FUSE_FSYNC => do_dispatch!(fsync),
		#[cfg(feature = "unstable_ioctl")]
		fuse_kernel::FUSE_IOCTL => do_dispatch!(ioctl),
		fuse_kernel::FUSE_OPEN => do_dispatch!(open),
		fuse_kernel::FUSE_READ => do_dispatch!(read),
		fuse_kernel::FUSE_RELEASE => do_dispatch!(release),
		fuse_kernel::FUSE_WRITE => do_dispatch!(write),
		_ => {
			#[cfg(feature = "std")]
			if let Some(hooks) = hooks {
				let request = UnknownRequest::decode_request(request_decoder)?;
				hooks.on_unknown(&request);
			}
			respond.encoder().encode_error(ErrorCode::ENOSYS)?;
		},
	}
	Ok(())
}
