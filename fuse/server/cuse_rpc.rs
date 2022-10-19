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

//! RPC-style CUSE servers.

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

#[allow(missing_docs)] // TODO
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

#[allow(missing_docs)] // TODO
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

#[allow(missing_docs)] // TODO
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
		let conn = server::CuseConnection::connect(
			self.socket,
			device_name,
			opts.device_number,
			|init_request, init_response| {
				init_response.set_max_read(opts.max_read);
				init_response.set_max_write(opts.max_write);
				init_response.set_flags(opts.flags);
				init_fn(init_request, init_response)
			},
		)?;

		Ok(Server {
			conn,
			handlers: self.handlers,
			#[cfg(feature = "std")]
			hooks: self.hooks,
		})
	}
}

// }}}

// Server {{{

/// An RPC-style CUSE device server.
///
/// Maintains ownership of a [`CuseConnection`] and a set of [`Handlers`]. Each
/// incoming request will be routed to the appropriate handler.
///
/// [`CuseConnection`]: server::CuseConnection
pub struct Server<S, H> {
	conn: server::CuseConnection<S>,
	handlers: H,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn server::Hooks>>,
}

impl<S, H> Server<S, H>
where
	S: CuseSocket,
	H: Handlers<S>,
{
	/// Serve CUSE requests in a loop.
	pub fn serve(&self) -> Result<core::convert::Infallible, ServerError<S::Error>> {
		let mut buf = crate::io::MinReadBuffer::new();

		#[allow(unused_mut)]
		let mut dispatcher = Dispatcher::new(&self.conn, &self.handlers);

		#[cfg(feature = "std")]
		if let Some(hooks) = self.hooks.as_ref() {
			dispatcher.set_hooks(hooks.as_ref());
		}

		loop {
			let request = self.conn.recv(buf.as_aligned_slice_mut())?;
			dispatcher.dispatch(request)?;
		}
	}
}

// }}}

// SendResult {{{

/// The result of sending a CUSE response.
///
/// Semantically this is a `Result<(), fuse::server::io::SendError<E>>`, but it
/// also serves as a marker to ensure that a CUSE handler can't return without
/// sending a response.
#[must_use]
pub struct SendResult<R, E> {
	_phantom: core::marker::PhantomData<fn(&R)>,
	error: Result<(), io::SendError<E>>,
}

impl<R, E> SendResult<R, E> {
	/// Returns `true` if the response was sent successfully.
	#[inline]
	#[must_use]
	pub fn is_ok(&self) -> bool {
		self.error.is_ok()
	}

	/// Returns `true` if the response was rejected by the client.
	#[inline]
	#[must_use]
	pub fn is_err(&self) -> bool {
		self.error.is_err()
	}

	/// Returns the underlying error if the response was rejected by the client.
	#[inline]
	#[must_use]
	pub fn err(&self) -> Option<&io::SendError<E>> {
		match self.error {
			Ok(_) => None,
			Err(ref err) => Some(err),
		}
	}
}

// }}}

// Call {{{

/// Represents a single call to an RPC-style CUSE handler.
pub struct Call<'a, S> {
	socket: &'a S,
	request: server::Request<'a>,
	response_options: server::CuseResponseOptions,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<S> Call<'_, S> {
	/// Returns the header of the underlying [`Request`].
	///
	/// [`Request`]: server::Request
	#[inline]
	#[must_use]
	pub fn header(&self) -> &crate::RequestHeader {
		self.request.header()
	}
}

impl<S: CuseSocket> Call<'_, S> {
	/// Sends a successful response to the CUSE client.
	pub fn respond_ok<R: server::CuseResponse>(
		self,
		response: &R,
	) -> SendResult<R, S::Error> {
		let mut response_header = crate::ResponseHeader::new(
			self.header().request_id(),
		);
		let error = self.socket.send(response.to_response(
			&mut response_header,
			self.response_options,
		).into());
		if let Err(ref err) = error {
			self.response_rejected(err);
		}
		SendResult {
			_phantom: core::marker::PhantomData,
			error,
		}
	}

	/// Sends an error response to the CUSE client.
	pub fn respond_err<R>(
		self,
		error: impl Into<crate::Error>,
	) -> SendResult<R, S::Error> {
		let request_id = self.header().request_id();
		let error = server::send_error(self.socket, request_id, error.into());
		SendResult {
			_phantom: core::marker::PhantomData,
			error,
		}
	}

	#[cold]
	fn response_rejected(self, err: &io::SendError<S::Error>) {
		if let io::SendError::NotFound(_) = err {
			return;
		}
		let request_id = self.header().request_id();
		let _ = server::send_error(
			self.socket,
			request_id,
			crate::Error::PROTOCOL_ERROR,
		);
	}

	fn unimplemented<R>(self) -> SendResult<R, S::Error> {
		if let Some(hooks) = self.hooks {
			hooks.unimplemented(self.request);
		}
		self.respond_err(crate::Error::UNIMPLEMENTED)
	}
}

// }}}

// Dispatcher {{{

/// Helper for dispatching CUSE requests to handlers.
pub struct Dispatcher<'a, S, H> {
	socket: &'a S,
	handlers: &'a H,
	request_options: server::CuseRequestOptions,
	response_options: server::CuseResponseOptions,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<'a, S, H> Dispatcher<'a, S, H> {
	/// Create a new `Dispatcher` for the given connection and handlers.
	pub fn new(
		conn: &'a server::CuseConnection<S>,
		handlers: &'a H,
	) -> Dispatcher<'a, S, H> {
		Self {
			socket: conn.socket(),
			handlers,
			request_options: conn.request_options(),
			response_options: conn.response_options(),
			hooks: None,
		}
	}

	/// Create a new `Dispatcher` with the given socket, handlers, and options.
	pub fn from_socket(
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

	/// Set optioal hooks for observing dispatch events.
	pub fn set_hooks(&mut self, hooks: &'a dyn server::Hooks) {
		self.hooks = Some(hooks);
	}
}

impl<S: CuseSocket, H: Handlers<S>> Dispatcher<'_, S, H> {
	/// Dispatch a single CUSE request.
	pub fn dispatch(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::Opcode;
		if let Some(hooks) = self.hooks {
			hooks.request(request);
		}
		let result = match request.header().opcode() {
			Opcode::FUSE_READ => self.do_read(request),
			Opcode::FUSE_WRITE => self.do_write(request),
			_ => self.do_other(request),
		};
		match result {
			Err(io::SendError::NotFound(_)) => Ok(()),
			_ => result,
		}
	}

	#[inline]
	fn new_call<'a>(
		&'a self,
		request: server::Request<'a>,
	) -> Call<'a, S> {
		Call {
			socket: self.socket,
			request,
			response_options: self.response_options,
			hooks: self.hooks,
		}
	}

	#[inline]
	fn do_read(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::CuseRequest;

		let call = self.new_call(request);
		match CuseRequest::from_request(request, self.request_options) {
			Ok(request) => self.handlers.read(call, &request).error,
			Err(err) => self.on_request_error(request, err),
		}
	}

	#[inline]
	fn do_write(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::CuseRequest;

		let call = self.new_call(request);
		match CuseRequest::from_request(request, self.request_options) {
			Ok(request) => self.handlers.write(call, &request).error,
			Err(err) => self.on_request_error(request, err),
		}
	}

	#[inline(never)]
	fn do_other(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::CuseRequest;
		use crate::Opcode as Op;

		let call = self.new_call(request);

		macro_rules! do_dispatch {
			($handler:tt) => {{
				match CuseRequest::from_request(request, self.request_options) {
					Ok(request) => self.handlers.$handler(call, &request).error,
					Err(err) => self.on_request_error(request, err),
				}
			}};
		}

		match request.header().opcode() {
			Op::FUSE_FLUSH => do_dispatch!(flush),
			Op::FUSE_FSYNC => do_dispatch!(fsync),
			Op::FUSE_INTERRUPT => {
				match CuseRequest::from_request(request, self.request_options) {
					Ok(request) => self.handlers.interrupt(call, &request),
					Err(err) => if let Some(hooks) = self.hooks {
						hooks.request_error(request, err);
					},
				};
				Ok(())
			},
			Op::FUSE_IOCTL => do_dispatch!(ioctl),
			Op::FUSE_OPEN => do_dispatch!(open),
			Op::FUSE_POLL => do_dispatch!(poll),
			Op::FUSE_RELEASE => do_dispatch!(release),
			_ => self.on_request_unknown(request),
		}
	}

	#[cold]
	#[inline(never)]
	fn on_request_error(
		&self,
		request: server::Request,
		err: server::RequestError,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::Error;
		use server::RequestError;

		if let Some(hooks) = self.hooks {
			hooks.request_error(request, err);
		}
		let request_id = request.header().request_id();
		server::send_error(self.socket, request_id, match err {
			RequestError::UnexpectedEof => Error::PROTOCOL_ERROR,
			RequestError::InvalidRequestId =>  Error::PROTOCOL_ERROR,
			_ =>  Error::INVALID_ARGUMENT,
		})
	}

	#[cold]
	#[inline(never)]
	fn on_request_unknown(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		if let Some(hooks) = self.hooks {
			hooks.unknown_opcode(request);
		}
		server::send_error(
			self.socket,
			request.header().request_id(),
			crate::Error::UNIMPLEMENTED,
		)
	}
}

// }}}

// Handlers {{{

/// RPC-style handlers for CUSE operations.
///
/// These handlers receive an operation-specific request value and a [`Call`]
/// containing metadata about the request. The call must be used to respond
/// by sending either an operation-specific response value or an error.
///
/// The default implementation for most handlers is to respond with
/// [`Error::UNIMPLEMENTED`]. If the [`Dispatcher`] has server hooks set, the
/// [`Hooks::unimplemented`] method will be called for each request received
/// by the default handler.
///
/// [`Error::UNIMPLEMENTED`]: crate::Error::UNIMPLEMENTED
/// [`Hooks::unimplemented`]: server::Hooks::unimplemented
#[allow(unused_variables)]
pub trait Handlers<S: CuseSocket> {
	/// Request handler for [`FUSE_FLUSH`].
	///
	/// See the [`fuse::operations::flush`] module for an overview of the
	/// `FUSE_FLUSH` operation.
	///
	/// [`FUSE_FLUSH`]: crate::Opcode::FUSE_FLUSH
	/// [`fuse::operations::flush`]: crate::operations::flush
	fn flush(
		&self,
		call: Call<S>,
		request: &operations::FlushRequest,
	) -> SendResult<operations::FlushResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_FSYNC`].
	///
	/// See the [`fuse::operations::fsync`] module for an overview of the
	/// `FUSE_FSYNC` operation.
	///
	/// [`FUSE_FSYNC`]: crate::Opcode::FUSE_FSYNC
	/// [`fuse::operations::fsync`]: crate::operations::fsync
	fn fsync(
		&self,
		call: Call<S>,
		request: &operations::FsyncRequest,
	) -> SendResult<operations::FsyncResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_INTERRUPT`].
	///
	/// See the [`fuse::operations::interrupt`] module for an overview of the
	/// `FUSE_INTERRUPT` operation.
	///
	/// [`FUSE_INTERRUPT`]: crate::Opcode::FUSE_INTERRUPT
	/// [`fuse::operations::interrupt`]: crate::operations::interrupt
	fn interrupt(
		&self,
		call: Call<S>,
		request: &operations::InterruptRequest,
	) {
		if let Some(hooks) = call.hooks {
			hooks.unimplemented(call.request);
		}
	}

	/// Request handler for [`FUSE_IOCTL`].
	///
	/// See the [`fuse::operations::ioctl`] module for an overview of the
	/// `FUSE_IOCTL` operation.
	///
	/// [`FUSE_IOCTL`]: crate::Opcode::FUSE_IOCTL
	/// [`fuse::operations::ioctl`]: crate::operations::ioctl
	fn ioctl(
		&self,
		call: Call<S>,
		request: &operations::IoctlRequest,
	) -> SendResult<operations::IoctlResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_OPEN`].
	///
	/// See the [`fuse::operations::open`] module for an overview of the
	/// `FUSE_OPEN` operation.
	///
	/// [`FUSE_OPEN`]: crate::Opcode::FUSE_OPEN
	/// [`fuse::operations::open`]: crate::operations::open
	fn open(
		&self,
		call: Call<S>,
		request: &operations::OpenRequest,
	) -> SendResult<operations::OpenResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_POLL`].
	///
	/// See the [`fuse::operations::poll`] module for an overview of the
	/// `FUSE_POLL` operation.
	///
	/// [`FUSE_POLL`]: crate::Opcode::FUSE_POLL
	/// [`fuse::operations::poll`]: crate::operations::poll
	fn poll(
		&self,
		call: Call<S>,
		request: &operations::PollRequest,
	) -> SendResult<operations::PollResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_READ`].
	///
	/// See the [`fuse::operations::read`] module for an overview of the
	/// `FUSE_READ` operation.
	///
	/// [`FUSE_READ`]: crate::Opcode::FUSE_READ
	/// [`fuse::operations::read`]: crate::operations::read
	fn read(
		&self,
		call: Call<S>,
		request: &operations::ReadRequest,
	) -> SendResult<operations::ReadResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_RELEASE`].
	///
	/// See the [`fuse::operations::release`] module for an overview of the
	/// `FUSE_RELEASE` operation.
	///
	/// [`FUSE_RELEASE`]: crate::Opcode::FUSE_RELEASE
	/// [`fuse::operations::release`]: crate::operations::release
	fn release(
		&self,
		call: Call<S>,
		request: &operations::ReleaseRequest,
	) -> SendResult<operations::ReleaseResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_WRITE`].
	///
	/// See the [`fuse::operations::write`] module for an overview of the
	/// `FUSE_WRITE` operation.
	///
	/// [`FUSE_WRITE`]: crate::Opcode::FUSE_WRITE
	/// [`fuse::operations::write`]: crate::operations::write
	fn write(
		&self,
		call: Call<S>,
		request: &operations::WriteRequest,
	) -> SendResult<operations::WriteResponse, S::Error> {
		call.unimplemented()
	}
}

// }}}
