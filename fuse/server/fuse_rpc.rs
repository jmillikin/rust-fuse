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

//! RPC-style FUSE servers.

#[cfg(any(doc, feature = "std"))]
use std::sync::mpsc;

use crate::operations;
use crate::server;
use crate::server::io;
use crate::server::ServerError;

pub use crate::server::io::FuseSocket;

// SendResult {{{

/// The result of sending a FUSE response.
///
/// Semantically this is a `Result<(), fuse::server::io::SendError<E>>`, but it
/// also serves as a marker to ensure that a FUSE handler can't return without
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

/// Represents a single call to an RPC-style FUSE handler.
pub struct Call<'a, S> {
	socket: &'a S,
	request: server::Request<'a>,
	response_options: server::FuseResponseOptions,
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

impl<S: FuseSocket> Call<'_, S> {
	/// Sends a successful response to the FUSE client.
	pub fn respond_ok<R: server::FuseResponse>(
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

	/// Sends an error response to the FUSE client.
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

/// Helper for dispatching FUSE requests to handlers.
pub struct Dispatcher<'a, S, H> {
	socket: &'a S,
	handlers: &'a H,
	request_options: server::FuseRequestOptions,
	response_options: server::FuseResponseOptions,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<'a, S, H> Dispatcher<'a, S, H> {
	/// Create a new `Dispatcher` for the given connection and handlers.
	pub fn new(
		conn: &'a server::FuseConnection<S>,
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
		request_options: server::FuseRequestOptions,
		response_options: server::FuseResponseOptions,
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

impl<S: FuseSocket, H: Handlers<S>> Dispatcher<'_, S, H> {
	/// Dispatch a single FUSE request.
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
		use crate::server::FuseRequest;

		let call = self.new_call(request);
		match FuseRequest::from_request(request, self.request_options) {
			Ok(request) => self.handlers.read(call, &request).error,
			Err(err) => self.on_request_error(request, err),
		}
	}

	#[inline]
	fn do_write(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::FuseRequest;

		let call = self.new_call(request);
		match FuseRequest::from_request(request, self.request_options) {
			Ok(request) => self.handlers.write(call, &request).error,
			Err(err) => self.on_request_error(request, err),
		}
	}

	#[inline(never)]
	fn do_other(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		use crate::server::FuseRequest;
		use crate::Opcode as Op;

		let call = self.new_call(request);

		macro_rules! do_dispatch {
			($handler:tt) => {{
				match FuseRequest::from_request(request, self.request_options) {
					Ok(request) => self.handlers.$handler(call, &request).error,
					Err(err) => self.on_request_error(request, err),
				}
			}};
		}

		match request.header().opcode() {
			Op::FUSE_ACCESS => do_dispatch!(access),
			Op::FUSE_BMAP => do_dispatch!(bmap),
			Op::FUSE_COPY_FILE_RANGE => do_dispatch!(copy_file_range),
			Op::FUSE_CREATE => do_dispatch!(create),
			Op::FUSE_DESTROY => do_dispatch!(destroy),
			Op::FUSE_FALLOCATE => do_dispatch!(fallocate),
			Op::FUSE_FLUSH => do_dispatch!(flush),
			Op::FUSE_FORGET | Op::FUSE_BATCH_FORGET => {
				match FuseRequest::from_request(request, self.request_options) {
					Ok(request) => self.handlers.forget(call, &request),
					Err(err) => if let Some(hooks) = self.hooks {
						hooks.request_error(request, err);
					},
				};
				Ok(())
			},
			Op::FUSE_FSYNC => do_dispatch!(fsync),
			Op::FUSE_FSYNCDIR => do_dispatch!(fsyncdir),
			Op::FUSE_GETATTR => do_dispatch!(getattr),
			Op::FUSE_GETLK => do_dispatch!(getlk),
			Op::FUSE_GETXATTR => do_dispatch!(getxattr),
			Op::FUSE_INTERRUPT => {
				match FuseRequest::from_request(request, self.request_options) {
					Ok(request) => self.handlers.interrupt(call, &request),
					Err(err) => if let Some(hooks) = self.hooks {
						hooks.request_error(request, err);
					},
				};
				Ok(())
			},
			Op::FUSE_IOCTL => do_dispatch!(ioctl),
			Op::FUSE_LINK => do_dispatch!(link),
			Op::FUSE_LISTXATTR => do_dispatch!(listxattr),
			Op::FUSE_LOOKUP => do_dispatch!(lookup),
			Op::FUSE_LSEEK => do_dispatch!(lseek),
			Op::FUSE_MKDIR => do_dispatch!(mkdir),
			Op::FUSE_MKNOD => do_dispatch!(mknod),
			Op::FUSE_OPEN => do_dispatch!(open),
			Op::FUSE_OPENDIR => do_dispatch!(opendir),
			Op::FUSE_POLL => do_dispatch!(poll),
			Op::FUSE_READDIR => do_dispatch!(readdir),
			Op::FUSE_READDIRPLUS => do_dispatch!(readdirplus),
			Op::FUSE_READLINK => do_dispatch!(readlink),
			Op::FUSE_RELEASE => do_dispatch!(release),
			Op::FUSE_RELEASEDIR => do_dispatch!(releasedir),
			Op::FUSE_REMOVEXATTR => do_dispatch!(removexattr),
			Op::FUSE_RENAME | Op::FUSE_RENAME2 => do_dispatch!(rename),
			Op::FUSE_RMDIR => do_dispatch!(rmdir),
			Op::FUSE_SETATTR => do_dispatch!(setattr),
			Op::FUSE_SETLK | Op::FUSE_SETLKW => do_dispatch!(setlk),
			Op::FUSE_SETXATTR => do_dispatch!(setxattr),
			Op::FUSE_STATFS => do_dispatch!(statfs),
			Op::FUSE_SYMLINK => do_dispatch!(symlink),
			Op::FUSE_SYNCFS => do_dispatch!(syncfs),
			Op::FUSE_UNLINK => do_dispatch!(unlink),
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

/// RPC-style handlers for FUSE operations.
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
/// On FreeBSD [`statfs`] returns an empty response by default, otherwise
/// mounting the filesystem will fail.
///
/// [`Error::UNIMPLEMENTED`]: crate::Error::UNIMPLEMENTED
/// [`Hooks::unimplemented`]: server::Hooks::unimplemented
/// [`statfs`]: Handlers::statfs
#[allow(unused_variables)]
pub trait Handlers<S: FuseSocket> {
	/// Request handler for [`FUSE_ACCESS`].
	///
	/// See the [`fuse::operations::access`] module for an overview of the
	/// `FUSE_ACCESS` operation.
	///
	/// [`FUSE_ACCESS`]: crate::Opcode::FUSE_ACCESS
	/// [`fuse::operations::access`]: crate::operations::access
	fn access(
		&self,
		call: Call<S>,
		request: &operations::AccessRequest,
	) -> SendResult<operations::AccessResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_BMAP`].
	///
	/// See the [`fuse::operations::bmap`] module for an overview of the
	/// `FUSE_BMAP` operation.
	///
	/// [`FUSE_BMAP`]: crate::Opcode::FUSE_BMAP
	/// [`fuse::operations::bmap`]: crate::operations::bmap
	fn bmap(
		&self,
		call: Call<S>,
		request: &operations::BmapRequest,
	) -> SendResult<operations::BmapResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_COPY_FILE_RANGE`].
	///
	/// See the [`fuse::operations::copy_file_range`] module for an overview
	/// of the `FUSE_COPY_FILE_RANGE` operation.
	///
	/// [`FUSE_COPY_FILE_RANGE`]: crate::Opcode::FUSE_COPY_FILE_RANGE
	/// [`fuse::operations::copy_file_range`]: crate::operations::copy_file_range
	fn copy_file_range(
		&self,
		call: Call<S>,
		request: &operations::CopyFileRangeRequest,
	) -> SendResult<operations::CopyFileRangeResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_CREATE`].
	///
	/// See the [`fuse::operations::create`] module for an overview of the
	/// `FUSE_CREATE` operation.
	///
	/// [`FUSE_CREATE`]: crate::Opcode::FUSE_CREATE
	/// [`fuse::operations::create`]: crate::operations::create
	fn create(
		&self,
		call: Call<S>,
		request: &operations::CreateRequest,
	) -> SendResult<operations::CreateResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_DESTROY`].
	///
	/// See the [`fuse::operations::destroy`] module for an overview of the
	/// `FUSE_DESTROY` operation.
	///
	/// [`FUSE_DESTROY`]: crate::Opcode::FUSE_DESTROY
	/// [`fuse::operations::destroy`]: crate::operations::destroy
	fn destroy(
		&self,
		call: Call<S>,
		request: &operations::DestroyRequest,
	) -> SendResult<operations::DestroyResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_FALLOCATE`].
	///
	/// See the [`fuse::operations::fallocate`] module for an overview of the
	/// `FUSE_FALLOCATE` operation.
	///
	/// [`FUSE_FALLOCATE`]: crate::Opcode::FUSE_FALLOCATE
	/// [`fuse::operations::fallocate`]: crate::operations::fallocate
	fn fallocate(
		&self,
		call: Call<S>,
		request: &operations::FallocateRequest,
	) -> SendResult<operations::FallocateResponse, S::Error> {
		call.unimplemented()
	}

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

	/// Request handler for [`FUSE_FORGET`] and [`FUSE_BATCH_FORGET`].
	///
	/// See the [`fuse::operations::forget`] module for an overview of the
	/// `FUSE_FORGET` and `FUSE_BATCH_FORGET` operations.
	///
	/// [`FUSE_FORGET`]: crate::Opcode::FUSE_FORGET
	/// [`FUSE_BATCH_FORGET`]: crate::Opcode::FUSE_BATCH_FORGET
	/// [`fuse::operations::forget`]: crate::operations::forget
	fn forget(&self, call: Call<S>, request: &operations::ForgetRequest) {
		if let Some(hooks) = call.hooks {
			hooks.unimplemented(call.request);
		}
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

	/// Request handler for [`FUSE_FSYNCDIR`].
	///
	/// See the [`fuse::operations::fsyncdir`] module for an overview of the
	/// `FUSE_FSYNCDIR` operation.
	///
	/// [`FUSE_FSYNCDIR`]: crate::Opcode::FUSE_FSYNCDIR
	/// [`fuse::operations::fsyncdir`]: crate::operations::fsyncdir
	fn fsyncdir(
		&self,
		call: Call<S>,
		request: &operations::FsyncdirRequest,
	) -> SendResult<operations::FsyncdirResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_GETATTR`].
	///
	/// See the [`fuse::operations::getattr`] module for an overview of the
	/// `FUSE_GETATTR` operation.
	///
	/// [`FUSE_GETATTR`]: crate::Opcode::FUSE_GETATTR
	/// [`fuse::operations::getattr`]: crate::operations::getattr
	fn getattr(
		&self,
		call: Call<S>,
		request: &operations::GetattrRequest,
	) -> SendResult<operations::GetattrResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_GETLK`].
	///
	/// See the [`fuse::operations::getlk`] module for an overview of the
	/// `FUSE_GETLK` operation.
	///
	/// [`FUSE_GETLK`]: crate::Opcode::FUSE_GETLK
	/// [`fuse::operations::getlk`]: crate::operations::getlk
	fn getlk(
		&self,
		call: Call<S>,
		request: &operations::GetlkRequest,
	) -> SendResult<operations::GetlkResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_GETXATTR`].
	///
	/// See the [`fuse::operations::getxattr`] module for an overview of the
	/// `FUSE_GETXATTR` operation.
	///
	/// [`FUSE_GETXATTR`]: crate::Opcode::FUSE_GETXATTR
	/// [`fuse::operations::getxattr`]: crate::operations::getxattr
	fn getxattr(
		&self,
		call: Call<S>,
		request: &operations::GetxattrRequest,
	) -> SendResult<operations::GetxattrResponse, S::Error> {
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

	/// Request handler for [`FUSE_LINK`].
	///
	/// See the [`fuse::operations::link`] module for an overview of the
	/// `FUSE_LINK` operation.
	///
	/// [`FUSE_LINK`]: crate::Opcode::FUSE_LINK
	/// [`fuse::operations::link`]: crate::operations::link
	fn link(
		&self,
		call: Call<S>,
		request: &operations::LinkRequest,
	) -> SendResult<operations::LinkResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_LISTXATTR`].
	///
	/// See the [`fuse::operations::listxattr`] module for an overview of the
	/// `FUSE_LISTXATTR` operation.
	///
	/// [`FUSE_LISTXATTR`]: crate::Opcode::FUSE_LISTXATTR
	/// [`fuse::operations::listxattr`]: crate::operations::listxattr
	fn listxattr(
		&self,
		call: Call<S>,
		request: &operations::ListxattrRequest,
	) -> SendResult<operations::ListxattrResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_LOOKUP`].
	///
	/// See the [`fuse::operations::lookup`] module for an overview of the
	/// `FUSE_LOOKUP` operation.
	///
	/// [`FUSE_LOOKUP`]: crate::Opcode::FUSE_LOOKUP
	/// [`fuse::operations::lookup`]: crate::operations::lookup
	fn lookup(
		&self,
		call: Call<S>,
		request: &operations::LookupRequest,
	) -> SendResult<operations::LookupResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_LSEEK`].
	///
	/// See the [`fuse::operations::lseek`] module for an overview of the
	/// `FUSE_LSEEK` operation.
	///
	/// [`FUSE_LSEEK`]: crate::Opcode::FUSE_LSEEK
	/// [`fuse::operations::lseek`]: crate::operations::lseek
	fn lseek(
		&self,
		call: Call<S>,
		request: &operations::LseekRequest,
	) -> SendResult<operations::LseekResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_MKDIR`].
	///
	/// See the [`fuse::operations::mkdir`] module for an overview of the
	/// `FUSE_MKDIR` operation.
	///
	/// [`FUSE_MKDIR`]: crate::Opcode::FUSE_MKDIR
	/// [`fuse::operations::mkdir`]: crate::operations::mkdir
	fn mkdir(
		&self,
		call: Call<S>,
		request: &operations::MkdirRequest,
	) -> SendResult<operations::MkdirResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_MKNOD`].
	///
	/// See the [`fuse::operations::mknod`] module for an overview of the
	/// `FUSE_MKNOD` operation.
	///
	/// [`FUSE_MKNOD`]: crate::Opcode::FUSE_MKNOD
	/// [`fuse::operations::mknod`]: crate::operations::mknod
	fn mknod(
		&self,
		call: Call<S>,
		request: &operations::MknodRequest,
	) -> SendResult<operations::MknodResponse, S::Error> {
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

	/// Request handler for [`FUSE_OPENDIR`].
	///
	/// See the [`fuse::operations::opendir`] module for an overview of the
	/// `FUSE_OPENDIR` operation.
	///
	/// [`FUSE_OPENDIR`]: crate::Opcode::FUSE_OPENDIR
	/// [`fuse::operations::opendir`]: crate::operations::opendir
	fn opendir(
		&self,
		call: Call<S>,
		request: &operations::OpendirRequest,
	) -> SendResult<operations::OpendirResponse, S::Error> {
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

	/// Request handler for [`FUSE_READDIR`].
	///
	/// See the [`fuse::operations::readdir`] module for an overview of the
	/// `FUSE_READDIR` operation.
	///
	/// [`FUSE_READDIR`]: crate::Opcode::FUSE_READDIR
	/// [`fuse::operations::readdir`]: crate::operations::readdir
	fn readdir(
		&self,
		call: Call<S>,
		request: &operations::ReaddirRequest,
	) -> SendResult<operations::ReaddirResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_READDIRPLUS`].
	///
	/// See the [`fuse::operations::readdirplus`] module for an overview of the
	/// `FUSE_READDIRPLUS` operation.
	///
	/// [`FUSE_READDIRPLUS`]: crate::Opcode::FUSE_READDIRPLUS
	/// [`fuse::operations::readdirplus`]: crate::operations::readdirplus
	fn readdirplus(
		&self,
		call: Call<S>,
		request: &operations::ReaddirplusRequest,
	) -> SendResult<operations::ReaddirplusResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_READLINK`].
	///
	/// See the [`fuse::operations::readlink`] module for an overview of the
	/// `FUSE_READLINK` operation.
	///
	/// [`FUSE_READLINK`]: crate::Opcode::FUSE_READLINK
	/// [`fuse::operations::readlink`]: crate::operations::readlink
	fn readlink(
		&self,
		call: Call<S>,
		request: &operations::ReadlinkRequest,
	) -> SendResult<operations::ReadlinkResponse, S::Error> {
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

	/// Request handler for [`FUSE_RELEASEDIR`].
	///
	/// See the [`fuse::operations::releasedir`] module for an overview of the
	/// `FUSE_RELEASEDIR` operation.
	///
	/// [`FUSE_RELEASEDIR`]: crate::Opcode::FUSE_RELEASEDIR
	/// [`fuse::operations::releasedir`]: crate::operations::releasedir
	fn releasedir(
		&self,
		call: Call<S>,
		request: &operations::ReleasedirRequest,
	) -> SendResult<operations::ReleasedirResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_REMOVEXATTR`].
	///
	/// See the [`fuse::operations::removexattr`] module for an overview of the
	/// `FUSE_REMOVEXATTR` operation.
	///
	/// [`FUSE_REMOVEXATTR`]: crate::Opcode::FUSE_REMOVEXATTR
	/// [`fuse::operations::removexattr`]: crate::operations::removexattr
	fn removexattr(
		&self,
		call: Call<S>,
		request: &operations::RemovexattrRequest,
	) -> SendResult<operations::RemovexattrResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_RENAME`] and [`FUSE_RENAME2`].
	///
	/// See the [`fuse::operations::rename`] module for an overview of the
	/// `FUSE_RENAME` and `FUSE_RENAME2` operations.
	///
	/// [`FUSE_RENAME`]: crate::Opcode::FUSE_RENAME
	/// [`FUSE_RENAME2`]: crate::Opcode::FUSE_RENAME2
	/// [`fuse::operations::rename`]: crate::operations::rename
	fn rename(
		&self,
		call: Call<S>,
		request: &operations::RenameRequest,
	) -> SendResult<operations::RenameResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_RMDIR`].
	///
	/// See the [`fuse::operations::rmdir`] module for an overview of the
	/// `FUSE_RMDIR` operation.
	///
	/// [`FUSE_RMDIR`]: crate::Opcode::FUSE_RMDIR
	/// [`fuse::operations::rmdir`]: crate::operations::rmdir
	fn rmdir(
		&self,
		call: Call<S>,
		request: &operations::RmdirRequest,
	) -> SendResult<operations::RmdirResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_SETATTR`].
	///
	/// See the [`fuse::operations::setattr`] module for an overview of the
	/// `FUSE_SETATTR` operation.
	///
	/// [`FUSE_SETATTR`]: crate::Opcode::FUSE_SETATTR
	/// [`fuse::operations::setattr`]: crate::operations::setattr
	fn setattr(
		&self,
		call: Call<S>,
		request: &operations::SetattrRequest,
	) -> SendResult<operations::SetattrResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_SETLK`] and [`FUSE_SETLKW`].
	///
	/// See the [`fuse::operations::setlk`] module for an overview of the
	/// `FUSE_SETLK` and `FUSE_SETLKW` operations.
	///
	/// [`FUSE_SETLK`]: crate::Opcode::FUSE_SETLK
	/// [`FUSE_SETLKW`]: crate::Opcode::FUSE_SETLKW
	/// [`fuse::operations::setlk`]: crate::operations::setlk
	fn setlk(
		&self,
		call: Call<S>,
		request: &operations::SetlkRequest,
	) -> SendResult<operations::SetlkResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_SETXATTR`].
	///
	/// See the [`fuse::operations::setxattr`] module for an overview of the
	/// `FUSE_SETXATTR` operation.
	///
	/// [`FUSE_SETXATTR`]: crate::Opcode::FUSE_SETXATTR
	/// [`fuse::operations::setxattr`]: crate::operations::setxattr
	fn setxattr(
		&self,
		call: Call<S>,
		request: &operations::SetxattrRequest,
	) -> SendResult<operations::SetxattrResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_STATFS`].
	///
	/// See the [`fuse::operations::statfs`] module for an overview of the
	/// `FUSE_STATFS` operation.
	///
	/// [`FUSE_STATFS`]: crate::Opcode::FUSE_STATFS
	/// [`fuse::operations::statfs`]: crate::operations::statfs
	fn statfs(
		&self,
		call: Call<S>,
		request: &operations::StatfsRequest,
	) -> SendResult<operations::StatfsResponse, S::Error> {
		#[cfg(not(target_os = "freebsd"))]
		{
			call.unimplemented()
		}

		#[cfg(target_os = "freebsd")]
		{
			if let Some(hooks) = call.hooks {
				hooks.unimplemented(call.request);
			}
			let resp = operations::StatfsResponse::new();
			call.respond_ok(&resp)
		}
	}

	/// Request handler for [`FUSE_SYMLINK`].
	///
	/// See the [`fuse::operations::symlink`] module for an overview of the
	/// `FUSE_SYMLINK` operation.
	///
	/// [`FUSE_SYMLINK`]: crate::Opcode::FUSE_SYMLINK
	/// [`fuse::operations::symlink`]: crate::operations::symlink
	fn symlink(
		&self,
		call: Call<S>,
		request: &operations::SymlinkRequest,
	) -> SendResult<operations::SymlinkResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_SYNCFS`].
	///
	/// See the [`fuse::operations::syncfs`] module for an overview of the
	/// `FUSE_SYNCFS` operation.
	///
	/// [`FUSE_SYNCFS`]: crate::Opcode::FUSE_SYNCFS
	/// [`fuse::operations::syncfs`]: crate::operations::syncfs
	fn syncfs(
		&self,
		call: Call<S>,
		request: &operations::SyncfsRequest,
	) -> SendResult<operations::SyncfsResponse, S::Error> {
		call.unimplemented()
	}

	/// Request handler for [`FUSE_UNLINK`].
	///
	/// See the [`fuse::operations::unlink`] module for an overview of the
	/// `FUSE_UNLINK` operation.
	///
	/// [`FUSE_UNLINK`]: crate::Opcode::FUSE_UNLINK
	/// [`fuse::operations::unlink`]: crate::operations::unlink
	fn unlink(
		&self,
		call: Call<S>,
		request: &operations::UnlinkRequest,
	) -> SendResult<operations::UnlinkResponse, S::Error> {
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

/// Serve FUSE requests in a loop.
///
/// This function spawns worker threads to process FUSE requests from the
/// given channel. The returned [`mpsc::Receiver`] can be used to receive
/// server errors from the worker threads, or dropped to run without error
/// reporting.
///
/// The worker threads will terminate if:
/// * An I/O error is reported by the socket.
/// * The connection is closed, such as by the user unmounting the filesystem
///   with `fusermount -u`.
///
/// # Panics
///
/// Panics on memory allocation failure. This function allocates
/// [`conn.recv_buf_len()`] bytes per worker thread, and also calls standard
/// library APIs such as [`Vec::with_capacity`] that panic on OOM.
///
/// [`conn.recv_buf_len()`]: server::FuseConnection::recv_buf_len
#[cfg(any(doc, feature = "std"))]
pub fn serve<S, H>(
	conn: &server::FuseConnection<S>,
	handlers: &H,
) -> mpsc::Receiver<ServerError<S::Error>>
where
	S: io::FuseSocket + Send + Sync,
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

/// Serve FUSE requests in a loop, in a single thread without allocating.
///
/// Returns `Ok(())` when the connection is closed, such as by the user
/// unmounting the filesystem with `fusermount -u`.
pub fn serve_local<S, H>(
	conn: &server::FuseConnection<S>,
	handlers: &H,
	buf: &mut impl crate::io::AsAlignedSliceMut,
) -> Result<(), ServerError<S::Error>>
where
	S: io::FuseSocket,
	H: Handlers<S>,
{
	let dispatcher = Dispatcher::new(conn, handlers);
	while let Some(request) = conn.recv(buf.as_aligned_slice_mut())? {
		dispatcher.dispatch(request)?;
	}
	Ok(())
}
