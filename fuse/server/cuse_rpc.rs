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
use crate::server::ServerError;

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

		let request_options =
			server::CuseRequestOptions::from_init_response(&init_response);
		let response_options =
			server::CuseResponseOptions::from_init_response(&init_response);
		Ok(Server {
			socket,
			handlers: self.handlers,
			request_options,
			response_options,
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
	request_options: server::CuseRequestOptions,
	response_options: server::CuseResponseOptions,

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
		let mut dispatcher = Dispatcher::new(
			&self.socket,
			&self.handlers,
			self.request_options,
			self.response_options,
		);

		#[cfg(feature = "std")]
		if let Some(hooks) = self.hooks.as_ref() {
			dispatcher.set_hooks(hooks.as_ref());
		}

		while let Some(request) = server::recv(
			&self.socket,
			buf.as_aligned_slice_mut(),
		)? {
			match dispatcher.dispatch(request) {
				Ok(()) => {},
				Err(io::SendError::NotFound(_)) => {},
				Err(err) => return Err(err.into()),
			};
		}
		Ok(())
	}
}

// }}}

// CuseResult {{{

mod sealed {
	pub struct Sent<T: ?Sized> {
		pub(super) _phantom: core::marker::PhantomData<fn(&T)>,
	}
}

pub type CuseResult<R, E> = Result<sealed::Sent<R>, io::SendError<E>>;

// }}}

// Call {{{

pub struct Call<'a, S> {
	socket: &'a S,
	header: &'a crate::RequestHeader,
	response_opts: server::CuseResponseOptions,
	sent_reply: &'a mut bool,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<S> Call<'_, S> {
	#[must_use]
	pub fn header(&self) -> &crate::RequestHeader {
		self.header
	}
}

impl<S: CuseSocket> Call<'_, S> {
	pub fn respond_ok<R: server::CuseResponse>(
		self,
		response: &R,
	) -> CuseResult<R, S::Error> {
		let mut response_header = crate::ResponseHeader::new(
			self.header.request_id(),
		);
		self.socket.send(response.to_response(
			&mut response_header,
			self.response_opts,
		).into())?;
		*self.sent_reply = true;
		Ok(sealed::Sent {
			_phantom: core::marker::PhantomData,
		})
	}

	pub fn respond_err<R>(
		self,
		error: impl Into<crate::Error>,
	) -> CuseResult<R, S::Error> {
		let request_id = self.header.request_id();
		server::send_error(self.socket, request_id, error.into())?;
		*self.sent_reply = true;
		Ok(sealed::Sent {
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

pub struct Dispatcher<'a, S, H> {
	socket: &'a S,
	handlers: &'a H,
	request_options: server::CuseRequestOptions,
	response_options: server::CuseResponseOptions,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<'a, S, H> Dispatcher<'a, S, H> {
	pub fn new(
		socket: &'a S,
		handlers: &'a H,
		request_options: server::CuseRequestOptions,
		response_options: server::CuseResponseOptions,
	) -> Dispatcher<'a, S, H> {
		Self {
			socket,
			handlers,
			request_options,
			response_options,
			hooks: None,
		}
	}

	pub fn set_hooks(&mut self, hooks: &'a dyn server::Hooks) {
		self.hooks = Some(hooks);
	}
}

impl<S: CuseSocket, H: Handlers<S>> Dispatcher<'_, S, H> {
	pub fn dispatch(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::Opcode;

		let header = request.header();
		if let Some(hooks) = self.hooks {
			hooks.request(header);
		}

		match header.opcode() {
			Opcode::FUSE_READ => self.do_read(request),
			Opcode::FUSE_WRITE => self.do_write(request),
			_ => self.do_other(request),
		}
	}

	#[inline]
	fn new_call<'a>(
		&'a self,
		header: &'a crate::RequestHeader,
		sent_reply: &'a mut bool,
	) -> Call<'a, S> {
		Call {
			socket: self.socket,
			header,
			response_opts: self.response_options,
			sent_reply,
			hooks: self.hooks,
		}
	}

	#[inline]
	fn do_read(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::CuseRequest;

		let header = request.header();
		let mut sent_reply = false;
		let call = self.new_call(header, &mut sent_reply);
		match CuseRequest::from_request(request, self.request_options) {
			Ok(request) => {
				let handler_result = self.handlers.read(call, &request);
				if !sent_reply {
					self.err_no_response(header);
				}
				handler_result?;
				Ok(())
			},
			Err(err) => self.on_request_error(header, err),
		}
	}

	#[inline]
	fn do_write(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::CuseRequest;

		let header = request.header();
		let mut sent_reply = false;
		let call = self.new_call(header, &mut sent_reply);
		match CuseRequest::from_request(request, self.request_options) {
			Ok(request) => {
				let handler_result = self.handlers.write(call, &request);
				if !sent_reply {
					self.err_no_response(header);
				}
				handler_result?;
				Ok(())
			},
			Err(err) => self.on_request_error(header, err),
		}
	}

	#[inline(never)]
	fn do_other(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::CuseRequest;
		use crate::Opcode as Op;

		let header = request.header();
		let mut sent_reply = false;
		let call = self.new_call(header, &mut sent_reply);

		macro_rules! do_dispatch {
			($handler:tt) => {{
				match CuseRequest::from_request(request, self.request_options) {
					Ok(request) => {
						let handler_result = self.handlers.$handler(call, &request);
						if !sent_reply {
							self.err_no_response(header);
						}
						handler_result?;
						Ok(())
					},
					Err(err) => self.on_request_error(header, err),
				}
			}};
		}

		match header.opcode() {
			Op::FUSE_FLUSH => do_dispatch!(flush),
			Op::FUSE_FSYNC => do_dispatch!(fsync),
			Op::FUSE_INTERRUPT => {
				match CuseRequest::from_request(request, self.request_options) {
					Ok(request) => self.handlers.interrupt(call, &request),
					Err(err) => if let Some(hooks) = self.hooks {
						hooks.request_error(header, err);
					},
				};
				Ok(())
			},
			Op::FUSE_IOCTL => do_dispatch!(ioctl),
			Op::FUSE_OPEN => do_dispatch!(open),
			Op::FUSE_POLL => do_dispatch!(poll),
			Op::FUSE_RELEASE => do_dispatch!(release),
			_ => self.on_request_unknown(header, request),
		}
	}

	#[cold]
	#[inline(never)]
	fn on_request_error(
		&self,
		header: &crate::RequestHeader,
		err: server::RequestError,
	) -> Result<(), io::SendError<S::Error>> {
		if let Some(hooks) = self.hooks {
			hooks.request_error(header, err);
		}
		server::send_error(
			self.socket,
			header.request_id(),
			crate::Error::INVALID_REQUEST,
		)
	}

	#[cold]
	#[inline(never)]
	fn on_request_unknown(
		&self,
		header: &crate::RequestHeader,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		if let Some(hooks) = self.hooks {
			let req = server::UnknownRequest::from_request(request);
			hooks.unknown_request(&req);
		}
		server::send_error(
			self.socket,
			header.request_id(),
			crate::Error::UNIMPLEMENTED,
		)
	}

	#[cold]
	#[inline(never)]
	fn err_no_response(&self, header: &crate::RequestHeader) {
		let _ = server::send_error(
			self.socket,
			header.request_id(),
			crate::Error::INVALID_REQUEST,
		);
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
