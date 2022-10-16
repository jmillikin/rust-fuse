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

use crate::operations;
use crate::operations::fuse_init::{
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server;
use crate::server::encode;
use crate::server::io;
use crate::server::ServerError;

pub use crate::server::io::FuseSocket;

// ServerBuilder {{{

pub struct ServerBuilder<S, H> {
	socket: S,
	handlers: H,
	opts: ServerOptions,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn server::Hooks>>,
}

struct ServerOptions {
	max_write: u32,
	flags: FuseInitFlags,
}

impl<S, H> ServerBuilder<S, H> {
	#[must_use]
	pub fn new(socket: S, handlers: H) -> Self {
		Self {
			socket,
			handlers,
			opts: ServerOptions {
				max_write: 0,
				flags: FuseInitFlags::new(),
			},
			#[cfg(feature = "std")]
			hooks: None,
		}
	}

	#[must_use]
	pub fn max_write(mut self, max_write: u32) -> Self {
		self.opts.max_write = max_write;
		self
	}

	#[must_use]
	pub fn fuse_init_flags(mut self, flags: FuseInitFlags) -> Self {
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

impl<S: FuseSocket, H> ServerBuilder<S, H> {
	pub fn fuse_init(self) -> Result<Server<S, H>, ServerError<S::Error>> {
		self.fuse_init_fn(|_init_request, _init_response| {})
	}

	pub fn fuse_init_fn(
		self,
		mut init_fn: impl FnMut(&FuseInitRequest, &mut FuseInitResponse),
	) -> Result<Server<S, H>, ServerError<S::Error>> {
		let opts = self.opts;
		let mut socket = self.socket;
		let init_response = server::fuse_init(&mut socket, |request| {
			let mut response = opts.init_response(request);
			init_fn(request, &mut response);
			response
		})?;

		let request_options =
			server::FuseRequestOptions::from_init_response(&init_response);
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
	fn init_response(
		&self,
		_request: &FuseInitRequest,
	) -> FuseInitResponse {
		let mut response = FuseInitResponse::new();
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
	request_options: server::FuseRequestOptions,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn server::Hooks>>,
}

impl<S, H> Server<S, H>
where
	S: FuseSocket,
	H: Handlers<S>,
{
	pub fn serve(&self) -> Result<(), ServerError<S::Error>> {
		let mut buf = crate::io::MinReadBuffer::new();

		#[allow(unused_mut)]
		let mut dispatcher = Dispatcher::new(
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

// FuseResult {{{

mod sealed {
	pub struct Sent<T: ?Sized> {
		pub(super) _phantom: core::marker::PhantomData<fn(&T)>,
	}
}

pub type FuseResult<R, E> = Result<sealed::Sent<R>, io::SendError<E>>;

// }}}

// Call {{{

pub struct Call<'a, S> {
	socket: &'a S,
	header: &'a crate::RequestHeader,
	response_header: &'a mut crate::ResponseHeader,
	response_opts: server::FuseResponseOptions,
	sent_reply: &'a mut bool,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<S> Call<'_, S> {
	#[must_use]
	pub fn header(&self) -> &crate::RequestHeader {
		self.header
	}
}

impl<S: FuseSocket> Call<'_, S> {
	pub fn respond_ok<R: server::FuseResponse>(
		self,
		response: &R,
	) -> FuseResult<R, S::Error> {
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
	) -> FuseResult<R, S::Error> {
		self.socket.send(encode::error(
			self.response_header,
			err.into(),
		).into())?;
		*self.sent_reply = true;
		Ok(sealed::Sent {
			_phantom: core::marker::PhantomData,
		})
	}

	fn unimplemented<R>(self) -> FuseResult<R, S::Error> {
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
	request_options: server::FuseRequestOptions,
	hooks: Option<&'a dyn server::Hooks>,
}

impl<'a, S, H> Dispatcher<'a, S, H> {
	pub fn new(
		socket: &'a S,
		handlers: &'a H,
		request_options: server::FuseRequestOptions,
	) -> Dispatcher<'a, S, H> {
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

impl<S: FuseSocket, H: Handlers<S>> Dispatcher<'_, S, H> {
	pub fn dispatch(
		&self,
		request: server::Request,
	) -> Result<(), io::SendError<S::Error>> {
		fuse_request_dispatch(
			self.socket,
			self.handlers,
			self.hooks,
			request,
			self.request_options,
		)
	}
}

fn fuse_request_dispatch<S: FuseSocket>(
	socket: &S,
	handlers: &impl Handlers<S>,
	hooks: Option<&dyn server::Hooks>,
	request: server::Request,
	opts: server::FuseRequestOptions,
) -> Result<(), io::SendError<S::Error>> {
	use crate::server::FuseRequest;

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
		response_opts: server::FuseResponseOptions {
			version_minor: opts.version_minor(),
		},
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
					let _ = err;
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
		Op::FUSE_ACCESS => do_dispatch!(AccessRequest, access),
		Op::FUSE_BMAP => do_dispatch!(BmapRequest, bmap),
		Op::FUSE_COPY_FILE_RANGE => {
			do_dispatch!(CopyFileRangeRequest, copy_file_range)
		},
		Op::FUSE_CREATE => do_dispatch!(CreateRequest, create),
		Op::FUSE_DESTROY => do_dispatch!(DestroyRequest, destroy),
		Op::FUSE_FALLOCATE => do_dispatch!(FallocateRequest, fallocate),
		Op::FUSE_FLUSH => do_dispatch!(FlushRequest, flush),
		Op::FUSE_FORGET | Op::FUSE_BATCH_FORGET => {
			match ForgetRequest::from_request(request, opts) {
				Ok(request) => handlers.forget(call, &request),
				Err(err) => {
					if let Some(hooks) = hooks {
						hooks.request_error(header, err);
					}
					let _ = err;
				},
			};
			Ok(())
		},
		Op::FUSE_FSYNC => do_dispatch!(FsyncRequest, fsync),
		Op::FUSE_FSYNCDIR => do_dispatch!(FsyncdirRequest, fsyncdir),
		Op::FUSE_GETATTR => do_dispatch!(GetattrRequest, getattr),
		Op::FUSE_GETLK => do_dispatch!(GetlkRequest, getlk),
		Op::FUSE_GETXATTR => do_dispatch!(GetxattrRequest, getxattr),
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
		Op::FUSE_LINK => do_dispatch!(LinkRequest, link),
		Op::FUSE_LISTXATTR => do_dispatch!(ListxattrRequest, listxattr),
		Op::FUSE_LOOKUP => do_dispatch!(LookupRequest, lookup),
		Op::FUSE_LSEEK => do_dispatch!(LseekRequest, lseek),
		Op::FUSE_MKDIR => do_dispatch!(MkdirRequest, mkdir),
		Op::FUSE_MKNOD => do_dispatch!(MknodRequest, mknod),
		Op::FUSE_OPEN => do_dispatch!(OpenRequest, open),
		Op::FUSE_OPENDIR => do_dispatch!(OpendirRequest, opendir),
		Op::FUSE_POLL => do_dispatch!(PollRequest, poll),
		Op::FUSE_READ => do_dispatch!(ReadRequest, read),
		Op::FUSE_READDIR => do_dispatch!(ReaddirRequest, readdir),
		Op::FUSE_READDIRPLUS => do_dispatch!(ReaddirplusRequest, readdirplus),
		Op::FUSE_READLINK => do_dispatch!(ReadlinkRequest, readlink),
		Op::FUSE_RELEASE => do_dispatch!(ReleaseRequest, release),
		Op::FUSE_RELEASEDIR => do_dispatch!(ReleasedirRequest, releasedir),
		Op::FUSE_REMOVEXATTR => do_dispatch!(RemovexattrRequest, removexattr),
		Op::FUSE_RENAME | Op::FUSE_RENAME2 => {
			do_dispatch!(RenameRequest, rename)
		},
		Op::FUSE_RMDIR => do_dispatch!(RmdirRequest, rmdir),
		Op::FUSE_SETATTR => do_dispatch!(SetattrRequest, setattr),
		Op::FUSE_SETLK | Op::FUSE_SETLKW => do_dispatch!(SetlkRequest, setlk),
		Op::FUSE_SETXATTR => do_dispatch!(SetxattrRequest, setxattr),
		Op::FUSE_STATFS => do_dispatch!(StatfsRequest, statfs),
		Op::FUSE_SYMLINK => do_dispatch!(SymlinkRequest, symlink),
		Op::FUSE_SYNCFS => do_dispatch!(SyncfsRequest, syncfs),
		Op::FUSE_UNLINK => do_dispatch!(UnlinkRequest, unlink),
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

/// User-provided handlers for FUSE operations.
///
/// Most FUSE handlers (with the exception of [`fuse_init`]) are asynchronous.
/// These handlers receive a [`ServerContext`] containing information about
/// the request itself, along with a [`ServerResponseWriter`] that must be used
/// to send the response.
///
/// The default implementation for all async handlers is to respond with
/// [`Error::UNIMPLEMENTED`].
///
/// [`fuse_init`]: #method.fuse_init
/// [`ServerContext`]: struct.ServerContext.html
/// [`ServerResponseWriter`]: struct.ServerResponseWriter.html
/// [`Error::UNIMPLEMENTED`]: crate::Error::UNIMPLEMENTED
#[allow(unused_variables)]
pub trait Handlers<S: FuseSocket> {
	fn access(
		&self,
		call: Call<S>,
		request: &operations::AccessRequest,
	) -> FuseResult<operations::AccessResponse, S::Error> {
		call.unimplemented()
	}

	fn bmap(
		&self,
		call: Call<S>,
		request: &operations::BmapRequest,
	) -> FuseResult<operations::BmapResponse, S::Error> {
		call.unimplemented()
	}

	fn copy_file_range(
		&self,
		call: Call<S>,
		request: &operations::CopyFileRangeRequest,
	) -> FuseResult<operations::CopyFileRangeResponse, S::Error> {
		call.unimplemented()
	}

	fn create(
		&self,
		call: Call<S>,
		request: &operations::CreateRequest,
	) -> FuseResult<operations::CreateResponse, S::Error> {
		call.unimplemented()
	}

	fn destroy(
		&self,
		call: Call<S>,
		request: &operations::DestroyRequest,
	) -> FuseResult<operations::DestroyResponse, S::Error> {
		call.unimplemented()
	}

	fn fallocate(
		&self,
		call: Call<S>,
		request: &operations::FallocateRequest,
	) -> FuseResult<operations::FallocateResponse, S::Error> {
		call.unimplemented()
	}

	fn flush(
		&self,
		call: Call<S>,
		request: &operations::FlushRequest,
	) -> FuseResult<operations::FlushResponse, S::Error> {
		call.unimplemented()
	}

	fn forget(&self, call: Call<S>, request: &operations::ForgetRequest) {
		if let Some(hooks) = call.hooks {
			hooks.unhandled_request(call.header);
		}
	}

	fn fsync(
		&self,
		call: Call<S>,
		request: &operations::FsyncRequest,
	) -> FuseResult<operations::FsyncResponse, S::Error> {
		call.unimplemented()
	}

	fn fsyncdir(
		&self,
		call: Call<S>,
		request: &operations::FsyncdirRequest,
	) -> FuseResult<operations::FsyncdirResponse, S::Error> {
		call.unimplemented()
	}

	fn getattr(
		&self,
		call: Call<S>,
		request: &operations::GetattrRequest,
	) -> FuseResult<operations::GetattrResponse, S::Error> {
		call.unimplemented()
	}

	fn getlk(
		&self,
		call: Call<S>,
		request: &operations::GetlkRequest,
	) -> FuseResult<operations::GetlkResponse, S::Error> {
		call.unimplemented()
	}

	fn getxattr(
		&self,
		call: Call<S>,
		request: &operations::GetxattrRequest,
	) -> FuseResult<operations::GetxattrResponse, S::Error> {
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
	) -> FuseResult<operations::IoctlResponse, S::Error> {
		call.unimplemented()
	}

	fn link(
		&self,
		call: Call<S>,
		request: &operations::LinkRequest,
	) -> FuseResult<operations::LinkResponse, S::Error> {
		call.unimplemented()
	}

	fn listxattr(
		&self,
		call: Call<S>,
		request: &operations::ListxattrRequest,
	) -> FuseResult<operations::ListxattrResponse, S::Error> {
		call.unimplemented()
	}

	fn lookup(
		&self,
		call: Call<S>,
		request: &operations::LookupRequest,
	) -> FuseResult<operations::LookupResponse, S::Error> {
		call.unimplemented()
	}

	fn lseek(
		&self,
		call: Call<S>,
		request: &operations::LseekRequest,
	) -> FuseResult<operations::LseekResponse, S::Error> {
		call.unimplemented()
	}

	fn mkdir(
		&self,
		call: Call<S>,
		request: &operations::MkdirRequest,
	) -> FuseResult<operations::MkdirResponse, S::Error> {
		call.unimplemented()
	}

	fn mknod(
		&self,
		call: Call<S>,
		request: &operations::MknodRequest,
	) -> FuseResult<operations::MknodResponse, S::Error> {
		call.unimplemented()
	}

	fn open(
		&self,
		call: Call<S>,
		request: &operations::OpenRequest,
	) -> FuseResult<operations::OpenResponse, S::Error> {
		call.unimplemented()
	}

	fn opendir(
		&self,
		call: Call<S>,
		request: &operations::OpendirRequest,
	) -> FuseResult<operations::OpendirResponse, S::Error> {
		call.unimplemented()
	}

	fn poll(
		&self,
		call: Call<S>,
		request: &operations::PollRequest,
	) -> FuseResult<operations::PollResponse, S::Error> {
		call.unimplemented()
	}

	fn read(
		&self,
		call: Call<S>,
		request: &operations::ReadRequest,
	) -> FuseResult<operations::ReadResponse, S::Error> {
		call.unimplemented()
	}

	fn readdir(
		&self,
		call: Call<S>,
		request: &operations::ReaddirRequest,
	) -> FuseResult<operations::ReaddirResponse, S::Error> {
		call.unimplemented()
	}

	fn readdirplus(
		&self,
		call: Call<S>,
		request: &operations::ReaddirplusRequest,
	) -> FuseResult<operations::ReaddirplusResponse, S::Error> {
		call.unimplemented()
	}

	fn readlink(
		&self,
		call: Call<S>,
		request: &operations::ReadlinkRequest,
	) -> FuseResult<operations::ReadlinkResponse, S::Error> {
		call.unimplemented()
	}

	fn release(
		&self,
		call: Call<S>,
		request: &operations::ReleaseRequest,
	) -> FuseResult<operations::ReleaseResponse, S::Error> {
		call.unimplemented()
	}

	fn releasedir(
		&self,
		call: Call<S>,
		request: &operations::ReleasedirRequest,
	) -> FuseResult<operations::ReleasedirResponse, S::Error> {
		call.unimplemented()
	}

	fn removexattr(
		&self,
		call: Call<S>,
		request: &operations::RemovexattrRequest,
	) -> FuseResult<operations::RemovexattrResponse, S::Error> {
		call.unimplemented()
	}

	fn rename(
		&self,
		call: Call<S>,
		request: &operations::RenameRequest,
	) -> FuseResult<operations::RenameResponse, S::Error> {
		call.unimplemented()
	}

	fn rmdir(
		&self,
		call: Call<S>,
		request: &operations::RmdirRequest,
	) -> FuseResult<operations::RmdirResponse, S::Error> {
		call.unimplemented()
	}

	fn setattr(
		&self,
		call: Call<S>,
		request: &operations::SetattrRequest,
	) -> FuseResult<operations::SetattrResponse, S::Error> {
		call.unimplemented()
	}

	fn setlk(
		&self,
		call: Call<S>,
		request: &operations::SetlkRequest,
	) -> FuseResult<operations::SetlkResponse, S::Error> {
		call.unimplemented()
	}

	fn setxattr(
		&self,
		call: Call<S>,
		request: &operations::SetxattrRequest,
	) -> FuseResult<operations::SetxattrResponse, S::Error> {
		call.unimplemented()
	}

	fn statfs(
		&self,
		call: Call<S>,
		request: &operations::StatfsRequest,
	) -> FuseResult<operations::StatfsResponse, S::Error> {
		#[cfg(not(target_os = "freebsd"))]
		{
			call.unimplemented()
		}

		#[cfg(target_os = "freebsd")]
		{
			if let Some(hooks) = call.hooks {
				hooks.unhandled_request(call.header);
			}
			let resp = operations::StatfsResponse::new();
			call.respond_ok(&resp)
		}
	}

	fn symlink(
		&self,
		call: Call<S>,
		request: &operations::SymlinkRequest,
	) -> FuseResult<operations::SymlinkResponse, S::Error> {
		call.unimplemented()
	}

	fn syncfs(
		&self,
		call: Call<S>,
		request: &operations::SyncfsRequest,
	) -> FuseResult<operations::SyncfsResponse, S::Error> {
		call.unimplemented()
	}

	fn unlink(
		&self,
		call: Call<S>,
		request: &operations::UnlinkRequest,
	) -> FuseResult<operations::UnlinkResponse, S::Error> {
		call.unimplemented()
	}

	fn write(
		&self,
		call: Call<S>,
		request: &operations::WriteRequest,
	) -> FuseResult<operations::WriteResponse, S::Error> {
		call.unimplemented()
	}
}

// }}}
