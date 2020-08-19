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
use std::sync::Arc;

use crate::channel::Channel;
use crate::cuse_handlers::CuseHandlers;
use crate::error::{Error, ErrorCode};
use crate::internal::fuse_io::{self, AlignedBuffer, DecodeRequest};
use crate::internal::fuse_kernel;
use crate::protocol;
use crate::server;

pub trait CuseServerChannel: Channel {
	fn try_clone(&self) -> Result<Self, Self::Error>;
}

// CuseServer {{{

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub struct CuseServer<Channel, Handlers> {
	channel: Arc<Channel>,
	handlers: Arc<Handlers>,
	fuse_version: crate::ProtocolVersion,
	read_buf_size: usize,
}

impl<C, H> CuseServer<C, H>
where
	C: CuseServerChannel,
	H: CuseHandlers,
{
	pub fn new(
		device_name: &CuseDeviceName,
		channel: C,
		mut handlers: H,
	) -> Result<CuseServer<C, H>, C::Error> {
		let init_response =
			Self::cuse_handshake(device_name, &channel, &mut handlers)?;
		let fuse_version = init_response.version();
		Ok(Self {
			channel: Arc::new(channel),
			handlers: Arc::new(handlers),
			fuse_version,
			read_buf_size: server::read_buf_size(init_response.max_write()),
		})
	}

	fn cuse_handshake(
		device_name: &CuseDeviceName,
		channel: &C,
		handlers: &mut H,
	) -> Result<protocol::CuseInitResponse, C::Error> {
		let mut read_buf = fuse_io::MinReadBuffer::new();

		loop {
			let request_size = channel.receive(read_buf.get_mut())?;
			let request_buf = fuse_io::aligned_slice(&read_buf, request_size);
			let request_decoder = fuse_io::RequestDecoder::new(
				request_buf,
				crate::ProtocolVersion::LATEST,
				fuse_io::Semantics::CUSE,
			)?;

			let request_header = request_decoder.header();
			if request_header.opcode != fuse_kernel::CUSE_INIT {
				return Err(
					Error::ExpectedCuseInit(request_header.opcode.0).into()
				);
			}

			let request_id = request_header.unique;
			let init_request =
				protocol::CuseInitRequest::decode_request(request_decoder)?;

			let encoder = fuse_io::ResponseEncoder::new(
				channel,
				request_id,
				// CuseInitResponse always encodes with its own version
				crate::ProtocolVersion::LATEST,
			);

			let major_version = init_request.version().major();
			if major_version != fuse_kernel::FUSE_KERNEL_VERSION {
				let init_response = protocol::CuseInitResponse::new(
					crate::ProtocolVersion::LATEST,
				);
				init_response.encode_response(encoder, None)?;
				continue;
			}

			let init_response = handlers.cuse_init(&init_request);
			init_response
				.encode_response(encoder, Some(device_name.as_bytes()))?;
			return Ok(init_response);
		}
	}

	pub fn run(&mut self) -> Result<(), C::Error>
	where
		C: Send + Sync + 'static,
	{
		cuse_main_loop::<C, H>(
			&self.channel,
			&*self.handlers,
			self.read_buf_size,
			self.fuse_version,
		)
	}

	#[cfg(any(doc, feature = "run_local"))]
	#[cfg_attr(doc, doc(cfg(feature = "run_local")))]
	pub fn run_local(&mut self) -> Result<(), C::Error> {
		cuse_main_loop::<C, H>(
			&self.channel,
			&*self.handlers,
			self.read_buf_size,
			self.fuse_version,
		)
	}

	pub fn new_executor(&self) -> Result<CuseServerExecutor<C, H>, C::Error> {
		let channel = self.channel.try_clone()?;
		Ok(CuseServerExecutor::new(
			Arc::new(channel),
			self.handlers.clone(),
			self.read_buf_size,
			self.fuse_version,
		))
	}
}

// }}}

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
		crate::protocol::common::DebugBytesAsString(&self.0).fmt(fmt)
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

// CuseServerExecutor {{{

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub struct CuseServerExecutor<C, H> {
	channel: Arc<C>,
	handlers: Arc<H>,
	read_buf_size: usize,
	fuse_version: crate::ProtocolVersion,
}

impl<C, H> CuseServerExecutor<C, H> {
	fn new(
		channel: Arc<C>,
		handlers: Arc<H>,
		read_buf_size: usize,
		fuse_version: crate::ProtocolVersion,
	) -> Self {
		Self {
			channel,
			handlers,
			read_buf_size,
			fuse_version,
		}
	}
}

impl<C, H> CuseServerExecutor<C, H>
where
	C: CuseServerChannel,
	H: CuseHandlers,
{
	pub fn run(&mut self) -> Result<(), C::Error>
	where
		C: Send + Sync + 'static,
	{
		cuse_main_loop::<C, H>(
			&self.channel,
			&*self.handlers,
			self.read_buf_size,
			self.fuse_version,
		)
	}

	#[cfg(any(doc, feature = "run_local"))]
	#[cfg_attr(doc, doc(cfg(feature = "run_local")))]
	pub fn run_local(&mut self) -> Result<(), C::Error> {
		cuse_main_loop::<C, H>(
			&self.channel,
			&*self.handlers,
			self.read_buf_size,
			self.fuse_version,
		)
	}
}

// }}}

#[cfg(not(feature = "run_local"))]
trait MaybeSendChannel {
	type T: CuseServerChannel + Send + Sync + 'static;
}

#[cfg(not(feature = "run_local"))]
impl<C> MaybeSendChannel for C
where
	C: CuseServerChannel + Send + Sync + 'static,
{
	type T = C;
}

#[cfg(feature = "run_local")]
trait MaybeSendChannel {
	type T: CuseServerChannel;
}

#[cfg(feature = "run_local")]
impl<C> MaybeSendChannel for C
where
	C: CuseServerChannel,
{
	type T = C;
}

fn cuse_main_loop<C, H>(
	channel: &Arc<C::T>,
	handlers: &H,
	read_buf_size: usize,
	fuse_version: crate::ProtocolVersion,
) -> Result<(), <<C as MaybeSendChannel>::T as Channel>::Error>
where
	C: MaybeSendChannel,
	H: CuseHandlers,
{
	let mut read_buf = fuse_io::AlignedVec::new(read_buf_size);
	loop {
		let request_size = match channel.receive(read_buf.get_mut()) {
			Err(err) => return Err(err),
			Ok(request_size) => request_size,
		};
		let request_buf = fuse_io::aligned_slice(&read_buf, request_size);
		let decoder = fuse_io::RequestDecoder::new(
			request_buf,
			fuse_version,
			fuse_io::Semantics::CUSE,
		)?;

		cuse_request_dispatch::<C, H>(handlers, decoder, &channel)?;
	}
}

fn cuse_request_dispatch<C, H>(
	handlers: &H,
	request_decoder: fuse_io::RequestDecoder,
	channel: &Arc<C::T>,
) -> Result<(), <<C as MaybeSendChannel>::T as Channel>::Error>
where
	C: MaybeSendChannel,
	H: CuseHandlers,
{
	let header = request_decoder.header();

	let fuse_version = request_decoder.version();
	let ctx = server::ServerContext::new(*header);

	let respond_once =
		server::RespondOnceImpl::new(channel, header.unique, fuse_version);

	macro_rules! do_dispatch {
		($handler:tt) => {{
			match DecodeRequest::decode_request(request_decoder) {
				Ok(request) => handlers.$handler(ctx, &request, respond_once),
				Err(err) => {
					// TODO: use ServerLogger to log the parse error
					let _ = err;
					respond_once.encoder().encode_error(ErrorCode::EIO)?;
				},
			}
		}};
	}

	match header.opcode {
		#[cfg(feature = "unstable_fuse_flush")]
		fuse_kernel::FUSE_FLUSH => do_dispatch!(flush),
		#[cfg(feature = "unstable_fuse_fsync")]
		fuse_kernel::FUSE_FSYNC => do_dispatch!(fsync),
		#[cfg(feature = "unstable_fuse_ioctl")]
		fuse_kernel::FUSE_IOCTL => do_dispatch!(ioctl),
		fuse_kernel::FUSE_OPEN => do_dispatch!(open),
		fuse_kernel::FUSE_READ => do_dispatch!(read),
		fuse_kernel::FUSE_RELEASE => do_dispatch!(release),
		fuse_kernel::FUSE_WRITE => do_dispatch!(write),
		_ => {
			let request =
				protocol::UnknownRequest::decode_request(request_decoder)?;
			// handlers.unknown(ctx, &request);
			// TODO: use ServerLogger to log the unknown request
			let _ = request;
			respond_once.encoder().encode_error(ErrorCode::ENOSYS)?;
		},
	}
	Ok(())
}
