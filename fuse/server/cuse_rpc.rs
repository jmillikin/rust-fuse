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

#[cfg(any(doc, feature = "std"))]
use std::sync::mpsc;

use crate::operations as ops;
use crate::server;
use crate::server::io;
use crate::server::ServerError;

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

impl<S: io::CuseSocket> Call<'_, S> {
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

impl<S: io::CuseSocket, H: Handlers<S>> Dispatcher<'_, S, H> {
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
pub trait Handlers<S: io::CuseSocket> {
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
		request: &ops::flush::FlushRequest,
	) -> SendResult<ops::flush::FlushResponse, S::Error> {
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
		request: &ops::fsync::FsyncRequest,
	) -> SendResult<ops::fsync::FsyncResponse, S::Error> {
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
		request: &ops::interrupt::InterruptRequest,
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
		request: &ops::ioctl::IoctlRequest,
	) -> SendResult<ops::ioctl::IoctlResponse, S::Error> {
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
		request: &ops::open::OpenRequest,
	) -> SendResult<ops::open::OpenResponse, S::Error> {
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
		request: &ops::poll::PollRequest,
	) -> SendResult<ops::poll::PollResponse, S::Error> {
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
		request: &ops::read::ReadRequest,
	) -> SendResult<ops::read::ReadResponse, S::Error> {
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
		request: &ops::release::ReleaseRequest,
	) -> SendResult<ops::release::ReleaseResponse, S::Error> {
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
		request: &ops::write::WriteRequest,
	) -> SendResult<ops::write::WriteResponse, S::Error> {
		call.unimplemented()
	}
}

// }}}

/// Serve CUSE requests in a loop.
///
/// This function spawns worker threads to process CUSE requests from the
/// given channel. The returned [`mpsc::Receiver`] can be used to receive
/// server errors from the worker threads, or dropped to run without error
/// reporting.
///
/// The worker threads will terminate if an I/O error is reported by the
/// socket.
///
/// # Panics
///
/// Panics on memory allocation failure. This function allocates
/// [`conn.recv_buf_len()`] bytes per worker thread, and also calls standard
/// library APIs such as [`Vec::with_capacity`] that panic on OOM.
///
/// [`conn.recv_buf_len()`]: server::CuseConnection::recv_buf_len
#[cfg(any(doc, feature = "std"))]
pub fn serve<S, H>(
	conn: &server::CuseConnection<S>,
	handlers: &H,
) -> mpsc::Receiver<ServerError<S::Error>>
where
	S: io::CuseSocket + Send + Sync,
	S::Error: Send,
	H: Handlers<S> + Send + Sync,
{
	use crate::io::AlignedBuf;

	// Use `thread::available_parallelism()` to estimate how many hardware
	// threads might be available. This number is clamped to 16 to avoid
	// allocating an unreasonable amount of memory on larger machines.
	//
	// It's expected that this estimate won't work for all possible servers,
	// either because it's too small (in a server doing lots of slow remote IO)
	// or too large (in a constrained environment). Since the `serve()` function
	// uses only public API, servers with special requirements can write their
	// own version with appropriate threadpool sizing.
	const MAX_THREADS: usize = 16;
	let num_threads = core::cmp::min(
		std::thread::available_parallelism().map_or(1, |n| n.get()),
		MAX_THREADS,
	);

	// Pre-allocate receive buffers so that an allocation failure will happen
	// before any server threads get spawned.
	let mut recv_bufs = Vec::with_capacity(num_threads);
	let recv_buf_len = conn.recv_buf_len();
	for _ii in 0..num_threads {
		#[allow(clippy::unwrap_used)]
		recv_bufs.push(AlignedBuf::with_capacity(recv_buf_len).unwrap());
	}

	let (err_sender, err_receiver) = mpsc::sync_channel(num_threads);
	std::thread::scope(|s| {
		for _ii in 0..num_threads {
			let err_sender = err_sender.clone();
			let mut buf = recv_bufs.remove(recv_bufs.len() - 1);
			s.spawn(move || {
				while let Err(err) = serve_local(conn, handlers, &mut buf) {
					let fatal = fatal_error(&err);
					let _ = err_sender.send(err);
					if fatal {
						return;
					}
				}
			});
		}
	});

	err_receiver
}

#[cfg(feature = "std")]
fn fatal_error<E>(err: &ServerError<E>) -> bool {
	match err {
		ServerError::RequestError(_) => false,
		_ => true,
	}
}

/// Serve CUSE requests in a loop, in a single thread without allocating.
pub fn serve_local<S: io::CuseSocket>(
	conn: &server::CuseConnection<S>,
	handlers: &impl Handlers<S>,
	buf: &mut impl crate::io::AsAlignedSliceMut,
) -> Result<(), ServerError<S::Error>> {
	let dispatcher = Dispatcher::new(conn, handlers);
	loop {
		let request = conn.recv(buf.as_aligned_slice_mut())?;
		dispatcher.dispatch(request)?;
	}
}
