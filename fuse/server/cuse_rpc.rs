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

use crate::io::ArrayBuffer;
use crate::protocol;
use crate::protocol::cuse_init::{
	CuseDeviceName,
	CuseInitFlags,
	CuseInitRequest,
	CuseInitResponse,
};
use crate::server;
use crate::server::io;
use crate::server::{CuseRequestBuilder, ErrorResponse, ServerError};

#[cfg(feature = "std")]
use crate::server::ServerHooks;

pub use crate::server::io::CuseSocket as CuseSocket;

pub struct CuseServerBuilder<S, H> {
	socket: S,
	handlers: H,
	opts: CuseOptions,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn ServerHooks>>,
}

struct CuseOptions {
	dev_major: u32,
	dev_minor: u32,
	max_read: u32,
	max_write: u32,
	flags: CuseInitFlags,
}

impl<S, H> CuseServerBuilder<S, H> {
	pub fn new(socket: S, handlers: H) -> Self {
		Self {
			socket,
			handlers,
			opts: CuseOptions {
				dev_major: 0,
				dev_minor: 0,
				max_read: 0,
				max_write: 0,
				flags: CuseInitFlags::new(),
			},
			#[cfg(feature = "std")]
			hooks: None,
		}
	}

	pub fn device_number(mut self, major: u32, minor: u32) -> Self {
		self.opts.dev_major = major;
		self.opts.dev_minor = minor;
		self
	}

	pub fn max_read(mut self, max_read: u32) -> Self {
		self.opts.max_read = max_read;
		self
	}

	pub fn max_write(mut self, max_write: u32) -> Self {
		self.opts.max_write = max_write;
		self
	}

	pub fn unrestricted_ioctl(mut self, unrestricted_ioctl: bool) -> Self {
		self.opts.flags.unrestricted_ioctl = unrestricted_ioctl;
		self
	}

	#[cfg(feature = "std")]
	pub fn server_hooks(mut self, hooks: Box<dyn ServerHooks>) -> Self {
		self.hooks = Some(hooks);
		self
	}
}

impl<S: CuseSocket, H> CuseServerBuilder<S, H> {
	pub fn cuse_init(
		self,
		device_name: &CuseDeviceName,
	) -> Result<CuseServer<S, H>, ServerError<S::Error>> {
		self.cuse_init_fn(device_name, |_init_request, _init_response| {})
	}

	pub fn cuse_init_fn(
		self,
		device_name: &CuseDeviceName,
		mut init_fn: impl FnMut(&CuseInitRequest, &mut CuseInitResponse),
	) -> Result<CuseServer<S, H>, ServerError<S::Error>> {
		let opts = self.opts;
		let mut socket = self.socket;
		let init_response = server::cuse_init(&mut socket, |request| {
			let mut response = opts.init_response(request, device_name);
			init_fn(request, &mut response);
			response
		})?;

		Ok(CuseServer {
			socket: socket,
			handlers: self.handlers,
			req_builder: CuseRequestBuilder::from_init_response(&init_response),
			#[cfg(feature = "std")]
			hooks: self.hooks,
		})
	}
}

impl CuseOptions {
	fn init_response<'a>(
		&self,
		_request: &CuseInitRequest,
		device_name: &'a CuseDeviceName,
	) -> CuseInitResponse<'a> {
		let mut response = CuseInitResponse::new(device_name);
		response.set_device_number(self.dev_major, self.dev_minor);
		response.set_max_read(self.max_read);
		response.set_max_write(self.max_write);
		*response.flags_mut() = self.flags;
		response
	}
}

pub struct CuseServer<S, H> {
	socket: S,
	handlers: H,
	req_builder: CuseRequestBuilder,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn ServerHooks>>,
}

impl<S, H> CuseServer<S, H>
where
	S: CuseSocket,
	H: CuseHandlers<S>,
{
	pub fn serve(&self) -> Result<(), ServerError<S::Error>> {
		let mut buf = ArrayBuffer::new();
		while let Some(request) = self.try_next(buf.borrow_mut())? {
			let result = cuse_request_dispatch(
				&self.socket,
				&self.handlers,
				#[cfg(feature = "std")]
				self.hooks.as_ref().map(|h| h.as_ref()),
				request,
			);
			match result {
				Ok(()) => {},
				Err(io::SendError::NotFound(_)) => {},
				Err(err) => return Err(err.into()),
			};
		}
		Ok(())
	}

	fn try_next<'a>(
		&self,
		buf: &'a mut [u8],
	) -> Result<Option<server::CuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = self.socket.recv(buf)?;
		Ok(Some(self.req_builder.build(&buf[..recv_len])?))
	}
}

mod sealed {
	pub struct Sent<T: ?Sized> {
		pub(super) _phantom: core::marker::PhantomData<fn(&T)>,
	}

	pub trait Sealed {
		fn __internal_send<S: super::CuseSocket>(
			&self,
			call: super::CuseCall<S>,
		) -> super::CuseResult<Self, S::Error>;
	}
}

use sealed::{Sealed, Sent};

pub type CuseResult<R, E> = Result<Sent<R>, io::SendError<E>>;

pub trait CuseResponse: Sealed {}

macro_rules! impl_cuse_response {
	( $( $t:ident $( , )? )+ ) => {
		$(
			impl CuseResponse for protocol::$t<'_> {}
			impl Sealed for protocol::$t<'_> {
				fn __internal_send<S: CuseSocket>(
					&self,
					call: CuseCall<S>,
				) -> CuseResult<Self, S::Error> {
					self.send(call.socket, &call.response_ctx)?;
					call.sent()
				}
			}
		)+
	}
}

impl_cuse_response! {
	FlushResponse,
	FsyncResponse,
	OpenResponse,
	ReadResponse,
	ReleaseResponse,
	WriteResponse,
}

#[cfg(any(doc, feature = "unstable_ioctl"))]
impl_cuse_response! { IoctlResponse }

pub struct CuseCall<'a, S> {
	socket: &'a S,
	header: &'a server::RequestHeader,
	response_ctx: server::ResponseContext,
	sent_reply: &'a mut bool,

	#[cfg(feature = "std")]
	hooks: Option<&'a dyn ServerHooks>,
}

impl<S> CuseCall<'_, S> {
	pub fn header(&self) -> &server::RequestHeader {
		self.header
	}

	pub fn response_context(&self) -> server::ResponseContext {
		self.response_ctx
	}
}

impl<S: CuseSocket> CuseCall<'_, S> {
	fn sent<T>(self) -> CuseResult<T, S::Error> {
		*self.sent_reply = true;
		Ok(Sent {
			_phantom: core::marker::PhantomData,
		})
	}
}

impl<S: CuseSocket> CuseCall<'_, S> {
	pub fn respond_ok<R: CuseResponse>(
		self,
		response: &R,
	) -> CuseResult<R, S::Error> {
		response.__internal_send(self)
	}

	pub fn respond_err<R>(
		self,
		err: impl Into<crate::Error>,
	) -> CuseResult<R, S::Error> {
		let response = ErrorResponse::new(err.into());
		response.send(self.socket, &self.response_ctx)?;
		*self.sent_reply = true;
		Ok(Sent {
			_phantom: core::marker::PhantomData,
		})
	}

	fn unimplemented<R>(self) -> CuseResult<R, S::Error> {
		#[cfg(feature = "std")]
		if let Some(hooks) = self.hooks {
			hooks.unhandled_request(self.header);
		}
		self.respond_err(crate::Error::UNIMPLEMENTED)
	}
}

fn cuse_request_dispatch<S: CuseSocket>(
	socket: &S,
	handlers: &impl CuseHandlers<S>,
	#[cfg(feature = "std")]
	hooks: Option<&dyn ServerHooks>,
	request: server::CuseRequest,
) -> Result<(), io::SendError<S::Error>> {
	let header = request.header();
	#[cfg(feature = "std")]
	if let Some(hooks) = hooks {
		hooks.request(header);
	}

	let response_ctx = request.response_context();

	let mut sent_reply = false;
	let call = CuseCall {
		socket,
		header,
		response_ctx,
		sent_reply: &mut sent_reply,
		#[cfg(feature = "std")]
		hooks,
	};

	macro_rules! do_dispatch {
		($req_type:ty, $handler:tt) => {{
			match <$req_type>::from_cuse_request(&request) {
				Ok(request) => {
					let handler_result = handlers.$handler(call, &request);
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
					#[cfg(feature = "std")]
					if let Some(ref hooks) = hooks {
						hooks.request_error(header, err);
					}
					let _ = err;
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
			#[cfg(feature = "std")]
			if let Some(hooks) = hooks {
				let req = server::UnknownRequest::from_cuse_request(&request);
				hooks.unknown_request(&req);
			}
			let response = ErrorResponse::new(crate::Error::UNIMPLEMENTED);
			response.send(socket, &response_ctx)
		},
	}
}

/// User-provided handlers for CUSE operations.
#[allow(unused_variables)]
pub trait CuseHandlers<S: CuseSocket> {
	fn flush(
		&self,
		call: CuseCall<S>,
		request: &protocol::FlushRequest,
	) -> CuseResult<protocol::FlushResponse, S::Error> {
		call.unimplemented()
	}

	fn fsync(
		&self,
		call: CuseCall<S>,
		request: &protocol::FsyncRequest,
	) -> CuseResult<protocol::FsyncResponse, S::Error> {
		call.unimplemented()
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	fn ioctl(
		&self,
		call: CuseCall<S>,
		request: &protocol::IoctlRequest,
	) -> CuseResult<protocol::IoctlResponse, S::Error> {
		call.unimplemented()
	}

	fn open(
		&self,
		call: CuseCall<S>,
		request: &protocol::OpenRequest,
	) -> CuseResult<protocol::OpenResponse, S::Error> {
		call.unimplemented()
	}

	fn read(
		&self,
		call: CuseCall<S>,
		request: &protocol::ReadRequest,
	) -> CuseResult<protocol::ReadResponse, S::Error> {
		call.unimplemented()
	}

	fn release(
		&self,
		call: CuseCall<S>,
		request: &protocol::ReleaseRequest,
	) -> CuseResult<protocol::ReleaseResponse, S::Error> {
		call.unimplemented()
	}

	fn write(
		&self,
		call: CuseCall<S>,
		request: &protocol::WriteRequest,
	) -> CuseResult<protocol::WriteResponse, S::Error> {
		call.unimplemented()
	}
}
