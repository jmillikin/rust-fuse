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

use crate::cuse;
use crate::operations;
use crate::operations::cuse_init::{
	CuseInitFlags,
	CuseInitRequest,
	CuseInitResponse,
};
use crate::server;
use crate::server::io;
use crate::server::{CuseRequestBuilder, ErrorResponse, ServerError};

pub use crate::server::io::CuseSocket;

// ServerBuilder {{{

pub struct ServerBuilder<S, H> {
	socket: S,
	handlers: H,
	opts: ServerOptions,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn server::Hooks>>,
}

struct ServerOptions {
	device_number: cuse::DeviceNumber,
	max_read: u32,
	max_write: u32,
	flags: CuseInitFlags,
}

impl<S, H> ServerBuilder<S, H> {
	#[must_use]
	pub fn new(socket: S, handlers: H) -> Self {
		Self {
			socket,
			handlers,
			opts: ServerOptions {
				device_number: cuse::DeviceNumber::new(0, 0),
				max_read: 0,
				max_write: 0,
				flags: CuseInitFlags::new(),
			},
			#[cfg(feature = "std")]
			hooks: None,
		}
	}

	#[must_use]
	pub fn device_number(mut self, device_number: cuse::DeviceNumber) -> Self {
		self.opts.device_number = device_number;
		self
	}

	#[must_use]
	pub fn max_read(mut self, max_read: u32) -> Self {
		self.opts.max_read = max_read;
		self
	}

	#[must_use]
	pub fn max_write(mut self, max_write: u32) -> Self {
		self.opts.max_write = max_write;
		self
	}

	#[must_use]
	pub fn cuse_init_flags(mut self, flags: CuseInitFlags) -> Self {
		self.opts.flags = flags;
		self
	}

	#[cfg(feature = "std")]
	#[must_use]
	pub fn server_hooks(mut self, hooks: Box<dyn server::Hooks>) -> Self {
		self.hooks = Some(hooks);
		self
	}
}

impl<S: CuseSocket, H> ServerBuilder<S, H> {
	pub fn cuse_init(
		self,
		device_name: &cuse::DeviceName,
	) -> Result<Server<S, H>, ServerError<S::Error>> {
		self.cuse_init_fn(device_name, |_init_request, _init_response| {})
	}

	pub fn cuse_init_fn(
		self,
		device_name: &cuse::DeviceName,
		mut init_fn: impl FnMut(&CuseInitRequest, &mut CuseInitResponse),
	) -> Result<Server<S, H>, ServerError<S::Error>> {
		let opts = self.opts;
		let mut socket = self.socket;
		let init_response = server::cuse_init(&mut socket, |request| {
			let mut response = opts.init_response(request, device_name);
			init_fn(request, &mut response);
			response
		})?;

		Ok(Server {
			socket,
			handlers: self.handlers,
			req_builder: CuseRequestBuilder::from_init_response(&init_response),
			#[cfg(feature = "std")]
			hooks: self.hooks,
		})
	}
}

impl ServerOptions {
	fn init_response<'a>(
		&self,
		_request: &CuseInitRequest,
		device_name: &'a cuse::DeviceName,
	) -> CuseInitResponse<'a> {
		let mut response = CuseInitResponse::new(device_name);
		response.set_device_number(self.device_number);
		response.set_max_read(self.max_read);
		response.set_max_write(self.max_write);
		response.set_flags(self.flags);
		response
	}
}

// }}}

// Server {{{

pub struct Server<S, H> {
	socket: S,
	handlers: H,
	req_builder: CuseRequestBuilder,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn server::Hooks>>,
}

impl<S, H> Server<S, H>
where
	S: CuseSocket,
	H: Handlers<S>,
{
	pub fn serve(&self) -> Result<(), ServerError<S::Error>> {
		let mut buf = crate::io::MinReadBuffer::new();

		#[allow(unused_mut)]
		let mut dispatcher = CuseDispatcher::new(&self.socket, &self.handlers);

		#[cfg(feature = "std")]
		if let Some(hooks) = self.hooks.as_ref() {
			dispatcher.set_hooks(hooks.as_ref());
		}

		while let Some(request) = self.try_next(&mut buf)? {
			match dispatcher.dispatch(&request) {
				Ok(()) => {},
				Err(io::SendError::NotFound(_)) => {},
				Err(err) => return Err(err.into()),
			};
		}
		Ok(())
	}

	fn try_next<'a>(
		&self,
		buf: &'a mut crate::io::MinReadBuffer,
	) -> Result<Option<server::CuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = self.socket.recv(buf.as_slice_mut())?;
		let recv_buf = buf.as_aligned_slice().truncate(recv_len);
		Ok(Some(self.req_builder.build(recv_buf)?))
	}
}

// }}}

// CuseResponse {{{

pub trait CuseResponse: Sealed {}

mod sealed {
	pub struct Sent<T: ?Sized> {
		pub(super) _phantom: core::marker::PhantomData<fn(&T)>,
	}

	pub trait Sealed {
		fn __internal_send<S: super::CuseSocket>(
			&self,
			call: super::Call<S>,
		) -> super::CuseResult<Self, S::Error>;
	}
}

use sealed::{Sealed, Sent};

pub type CuseResult<R, E> = Result<Sent<R>, io::SendError<E>>;

macro_rules! impl_cuse_response {
	( $( $t:ident $( , )? )+ ) => {
		$(
			impl CuseResponse for operations::$t<'_> {}
			impl Sealed for operations::$t<'_> {
				fn __internal_send<S: CuseSocket>(
					&self,
					call: Call<S>,
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
	IoctlResponse,
	OpenResponse,
	ReadResponse,
	ReleaseResponse,
	WriteResponse,
}

// }}}

// Call {{{

pub struct Call<'a, S> {
	socket: &'a S,
	header: &'a crate::RequestHeader,
	response_ctx: server::ResponseContext,
	sent_reply: &'a mut bool,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<S> Call<'_, S> {
	#[must_use]
	pub fn header(&self) -> &crate::RequestHeader {
		self.header
	}

	#[must_use]
	pub fn response_context(&self) -> server::ResponseContext {
		self.response_ctx
	}
}

impl<S: CuseSocket> Call<'_, S> {
	fn sent<T>(self) -> CuseResult<T, S::Error> {
		*self.sent_reply = true;
		Ok(Sent {
			_phantom: core::marker::PhantomData,
		})
	}
}

impl<S: CuseSocket> Call<'_, S> {
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
		if let Some(hooks) = self.hooks {
			hooks.unhandled_request(self.header);
		}
		self.respond_err(crate::Error::UNIMPLEMENTED)
	}
}

// }}}

// Dispatcher {{{

pub struct CuseDispatcher<'a, S, H> {
	socket: &'a S,
	handlers: &'a H,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<'a, S, H> CuseDispatcher<'a, S, H> {
	pub fn new(socket: &'a S, handlers: &'a H) -> CuseDispatcher<'a, S, H> {
		Self {
			socket,
			handlers,
			hooks: None,
		}
	}

	pub fn set_hooks(&mut self, hooks: &'a dyn server::Hooks) {
		self.hooks = Some(hooks);
	}
}

impl<S: CuseSocket, H: Handlers<S>> CuseDispatcher<'_, S, H> {
	pub fn dispatch(
		&self,
		request: &server::CuseRequest,
	) -> Result<(), io::SendError<S::Error>> {
		cuse_request_dispatch(self.socket, self.handlers, self.hooks, request)
	}
}

fn cuse_request_dispatch<S: CuseSocket>(
	socket: &S,
	handlers: &impl Handlers<S>,
	hooks: Option<&dyn server::Hooks>,
	request: &server::CuseRequest,
) -> Result<(), io::SendError<S::Error>> {
	use crate::server::decode::CuseRequest;

	let header = request.header();
	if let Some(hooks) = hooks {
		hooks.request(header);
	}

	let response_ctx = request.response_context();

	let mut sent_reply = false;
	let call = Call {
		socket,
		header,
		response_ctx,
		sent_reply: &mut sent_reply,
		hooks,
	};

	macro_rules! do_dispatch {
		($req_type:ty, $handler:tt) => {{
			match <$req_type>::from_cuse_request(request) {
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
					if let Some(hooks) = hooks {
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
	use crate::operations::*;
	match request.header().opcode() {
		Op::FUSE_FLUSH => do_dispatch!(FlushRequest, flush),
		Op::FUSE_FSYNC => do_dispatch!(FsyncRequest, fsync),
		Op::FUSE_INTERRUPT => {
			match InterruptRequest::from_cuse_request(request) {
				Ok(request) => handlers.interrupt(call, &request),
				Err(err) => {
					if let Some(hooks) = hooks {
						hooks.request_error(header, err);
					}
					let _ = err;
				},
			};
			Ok(())
		},
		Op::FUSE_IOCTL => do_dispatch!(IoctlRequest, ioctl),
		Op::FUSE_OPEN => do_dispatch!(OpenRequest, open),
		Op::FUSE_POLL => do_dispatch!(PollRequest, poll),
		Op::FUSE_READ => do_dispatch!(ReadRequest, read),
		Op::FUSE_RELEASE => do_dispatch!(ReleaseRequest, release),
		Op::FUSE_WRITE => do_dispatch!(WriteRequest, write),
		_ => {
			if let Some(hooks) = hooks {
				let req = server::UnknownRequest::from_cuse_request(request);
				hooks.unknown_request(&req);
			}
			let response = ErrorResponse::new(crate::Error::UNIMPLEMENTED);
			response.send(socket, &response_ctx)
		},
	}
}

// }}}

// Handlers {{{

/// User-provided handlers for CUSE operations.
#[allow(unused_variables)]
pub trait Handlers<S: CuseSocket> {
	fn flush(
		&self,
		call: Call<S>,
		request: &operations::FlushRequest,
	) -> CuseResult<operations::FlushResponse, S::Error> {
		call.unimplemented()
	}

	fn fsync(
		&self,
		call: Call<S>,
		request: &operations::FsyncRequest,
	) -> CuseResult<operations::FsyncResponse, S::Error> {
		call.unimplemented()
	}

	fn interrupt(
		&self,
		call: Call<S>,
		request: &operations::InterruptRequest,
	) {
		if let Some(hooks) = call.hooks {
			hooks.unhandled_request(call.header);
		}
	}

	fn ioctl(
		&self,
		call: Call<S>,
		request: &operations::IoctlRequest,
	) -> CuseResult<operations::IoctlResponse, S::Error> {
		call.unimplemented()
	}

	fn open(
		&self,
		call: Call<S>,
		request: &operations::OpenRequest,
	) -> CuseResult<operations::OpenResponse, S::Error> {
		call.unimplemented()
	}

	fn poll(
		&self,
		call: Call<S>,
		request: &operations::PollRequest,
	) -> CuseResult<operations::PollResponse, S::Error> {
		call.unimplemented()
	}

	fn read(
		&self,
		call: Call<S>,
		request: &operations::ReadRequest,
	) -> CuseResult<operations::ReadResponse, S::Error> {
		call.unimplemented()
	}

	fn release(
		&self,
		call: Call<S>,
		request: &operations::ReleaseRequest,
	) -> CuseResult<operations::ReleaseResponse, S::Error> {
		call.unimplemented()
	}

	fn write(
		&self,
		call: Call<S>,
		request: &operations::WriteRequest,
	) -> CuseResult<operations::WriteResponse, S::Error> {
		call.unimplemented()
	}
}

// }}}
