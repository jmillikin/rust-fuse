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
use crate::server::encode;
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
		Ok(Server {
			socket,
			handlers: self.handlers,
			request_options,
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
		let mut dispatcher = CuseDispatcher::new(
			&self.socket,
			&self.handlers,
			self.request_options,
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
	response_header: &'a mut crate::ResponseHeader,
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
		self.socket.send(response.to_response(
			self.response_header,
			self.response_opts,
		).into())?;
		*self.sent_reply = true;
		Ok(sealed::Sent {
			_phantom: core::marker::PhantomData,
		})
	}

	pub fn respond_err<R>(
		self,
		err: impl Into<crate::Error>,
	) -> CuseResult<R, S::Error> {
		self.socket.send(encode::error(
			self.response_header,
			err.into(),
		).into())?;
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

pub struct CuseDispatcher<'a, S, H> {
	socket: &'a S,
	handlers: &'a H,
	request_options: server::CuseRequestOptions,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<'a, S, H> CuseDispatcher<'a, S, H> {
	pub fn new(
		socket: &'a S,
		handlers: &'a H,
		request_options: server::CuseRequestOptions,
	) -> CuseDispatcher<'a, S, H> {
		Self {
			socket,
			handlers,
			request_options,
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
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		cuse_request_dispatch(
			self.socket,
			self.handlers,
			self.hooks,
			request,
			self.request_options,
		)
	}
}

fn cuse_request_dispatch<S: CuseSocket>(
	socket: &S,
	handlers: &impl Handlers<S>,
	hooks: Option<&dyn server::Hooks>,
	request: server::Request,
	opts: server::CuseRequestOptions,
) -> Result<(), io::SendError<S::Error>> {
	use crate::server::CuseRequest;

	let header = request.header();
	if let Some(hooks) = hooks {
		hooks.request(header);
	}

	let mut resp_header = crate::ResponseHeader::new(header.request_id());
	let mut sent_reply = false;
	let call = Call {
		socket,
		header,
		response_header: &mut resp_header,
		response_opts: server::CuseResponseOptions { _empty: () },
		sent_reply: &mut sent_reply,
		hooks,
	};

	macro_rules! do_dispatch {
		($req_type:ty, $handler:tt) => {{
			match <$req_type>::from_request(request, opts) {
				Ok(request) => {
					let handler_result = handlers.$handler(call, &request);
					if sent_reply {
						handler_result?;
					} else {
						let err_result = socket.send(encode::error(
							&mut resp_header,
							crate::Error::EIO,
						).into());
						handler_result?;
						err_result?;
					}
					Ok(())
				},
				Err(err) => {
					if let Some(hooks) = hooks {
						hooks.request_error(header, err);
					}
					socket.send(encode::error(
						&mut resp_header,
						crate::Error::EIO,
					).into())
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
			match InterruptRequest::from_request(request, opts) {
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
				let req = server::UnknownRequest::from_request(request);
				hooks.unknown_request(&req);
			}
			socket.send(encode::error(
				&mut resp_header,
				crate::Error::UNIMPLEMENTED,
			).into())
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
