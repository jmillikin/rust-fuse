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
use crate::protocol::fuse_init::{
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server;
use crate::server::io;
use crate::server::{ErrorResponse, FuseRequestBuilder, ServerError};

#[cfg(feature = "std")]
use crate::server::ServerHooks;

pub use crate::server::io::FuseSocket as FuseSocket;

pub struct FuseServerBuilder<S, H> {
	socket: S,
	handlers: H,
	opts: FuseOptions,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn ServerHooks>>,
}

struct FuseOptions {
	max_write: u32,
	flags: FuseInitFlags,
}

impl<S, H> FuseServerBuilder<S, H> {
	pub fn new(socket: S, handlers: H) -> Self {
		Self {
			socket,
			handlers,
			opts: FuseOptions {
				max_write: 0,
				flags: FuseInitFlags::new(),
			},
			#[cfg(feature = "std")]
			hooks: None,
		}
	}

	pub fn max_write(mut self, max_write: u32) -> Self {
		self.opts.max_write = max_write;
		self
	}

	#[cfg(feature = "std")]
	pub fn server_hooks(mut self, hooks: Box<dyn ServerHooks>) -> Self {
		self.hooks = Some(hooks);
		self
	}
}

impl<S: FuseSocket, H> FuseServerBuilder<S, H> {
	pub fn fuse_init(self) -> Result<FuseServer<S, H>, ServerError<S::Error>> {
		self.fuse_init_fn(|_init_request, _init_response| {})
	}

	pub fn fuse_init_fn(
		self,
		mut init_fn: impl FnMut(&FuseInitRequest, &mut FuseInitResponse),
	) -> Result<FuseServer<S, H>, ServerError<S::Error>> {
		let opts = self.opts;
		let mut socket = self.socket;
		let init_response = server::fuse_init(&mut socket, |request| {
			let mut response = opts.init_response(request);
			init_fn(request, &mut response);
			response
		})?;

		Ok(FuseServer {
			socket: socket,
			handlers: self.handlers,
			req_builder: FuseRequestBuilder::from_init_response(&init_response),
			#[cfg(feature = "std")]
			hooks: self.hooks,
		})
	}
}

impl FuseOptions {
	fn init_response<'a>(
		&self,
		_request: &FuseInitRequest,
	) -> FuseInitResponse {
		let mut response = FuseInitResponse::new();
		response.set_max_write(self.max_write);
		*response.flags_mut() = self.flags;
		response
	}
}

pub struct FuseServer<S, H> {
	socket: S,
	handlers: H,
	req_builder: FuseRequestBuilder,

	#[cfg(feature = "std")]
	hooks: Option<Box<dyn ServerHooks>>,
}

impl<S, H> FuseServer<S, H>
where
	S: FuseSocket,
	H: FuseHandlers<S>,
{
	pub fn serve(&self) -> Result<(), ServerError<S::Error>> {
		let mut buf = ArrayBuffer::new();
		while let Some(request) = self.try_next(buf.borrow_mut())? {
			let result = fuse_request_dispatch(
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
	) -> Result<Option<server::FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf) {
			Ok(x) => x,
			Err(io::RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(err) => return Err(err.into()),
		};
		Ok(Some(self.req_builder.build(&buf[..recv_len])?))
	}
}

mod sealed {
	pub struct Sent<T: ?Sized> {
		pub(super) _phantom: core::marker::PhantomData<fn(&T)>,
	}

	pub trait Sealed {
		fn __internal_send<S: super::FuseSocket>(
			&self,
			call: super::FuseCall<S>,
		) -> super::FuseResult<Self, S::Error>;
	}
}

use sealed::{Sealed, Sent};

pub type FuseResult<R, E> = Result<Sent<R>, io::SendError<E>>;

pub trait FuseResponse: Sealed {}

macro_rules! impl_fuse_response {
	( $( $t:ident $( , )? )+ ) => {
		$(
			impl FuseResponse for protocol::$t<'_> {}
			impl Sealed for protocol::$t<'_> {
				fn __internal_send<S: FuseSocket>(
					&self,
					call: FuseCall<S>,
				) -> FuseResult<Self, S::Error> {
					self.send(call.socket, &call.response_ctx)?;
					call.sent()
				}
			}
		)+
	}
}

impl_fuse_response! {
	AccessResponse,
	CopyFileRangeResponse,
	CreateResponse,
	FallocateResponse,
	FlushResponse,
	FsyncdirResponse,
	FsyncResponse,
	GetattrResponse,
	GetlkResponse,
	GetxattrResponse,
	LinkResponse,
	ListxattrResponse,
	LookupResponse,
	LseekResponse,
	MkdirResponse,
	MknodResponse,
	OpendirResponse,
	OpenResponse,
	ReaddirResponse,
	ReadlinkResponse,
	ReadResponse,
	ReleasedirResponse,
	ReleaseResponse,
	RemovexattrResponse,
	RenameResponse,
	RmdirResponse,
	SetlkResponse,
	SetxattrResponse,
	StatfsResponse,
	SymlinkResponse,
	UnlinkResponse,
	WriteResponse,
}

#[cfg(any(doc, feature = "unstable_bmap"))]
impl_fuse_response! { BmapResponse }

#[cfg(any(doc, feature = "unstable_ioctl"))]
impl_fuse_response! { IoctlResponse }

#[cfg(any(doc, feature = "unstable_setattr"))]
impl_fuse_response! { SetattrResponse }

pub struct FuseCall<'a, S> {
	socket: &'a S,
	header: &'a server::RequestHeader,
	response_ctx: server::ResponseContext,
	sent_reply: &'a mut bool,

	#[cfg(feature = "std")]
	hooks: Option<&'a dyn ServerHooks>,
}

impl<S> FuseCall<'_, S> {
	pub fn header(&self) -> &server::RequestHeader {
		self.header
	}

	pub fn response_context(&self) -> server::ResponseContext {
		self.response_ctx
	}
}

impl<S: FuseSocket> FuseCall<'_, S> {
	fn sent<T>(self) -> FuseResult<T, S::Error> {
		*self.sent_reply = true;
		Ok(Sent {
			_phantom: core::marker::PhantomData,
		})
	}
}

impl<S: FuseSocket> FuseCall<'_, S> {
	pub fn respond_ok<R: FuseResponse>(
		self,
		response: &R,
	) -> FuseResult<R, S::Error> {
		response.__internal_send(self)
	}

	pub fn respond_err<R>(
		self,
		err: impl Into<crate::Error>,
	) -> FuseResult<R, S::Error> {
		let response = ErrorResponse::new(err.into());
		response.send(self.socket, &self.response_ctx)?;
		*self.sent_reply = true;
		Ok(Sent {
			_phantom: core::marker::PhantomData,
		})
	}

	fn unimplemented<R>(self) -> FuseResult<R, S::Error> {
		#[cfg(feature = "std")]
		if let Some(hooks) = self.hooks {
			hooks.unhandled_request(self.header);
		}
		self.respond_err(crate::Error::UNIMPLEMENTED)
	}
}

fn fuse_request_dispatch<S: FuseSocket>(
	socket: &S,
	handlers: &impl FuseHandlers<S>,
	#[cfg(feature = "std")]
	hooks: Option<&dyn ServerHooks>,
	request: server::FuseRequest,
) -> Result<(), io::SendError<S::Error>> {
	let header = request.header();
	#[cfg(feature = "std")]
	if let Some(hooks) = hooks {
		hooks.request(header);
	}

	let response_ctx = request.response_context();

	let mut sent_reply = false;
	let call = FuseCall {
		socket,
		header,
		response_ctx,
		sent_reply: &mut sent_reply,
		#[cfg(feature = "std")]
		hooks,
	};

	macro_rules! do_dispatch {
		($req_type:ty, $handler:tt) => {{
			match <$req_type>::from_fuse_request(&request) {
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
		Op::FUSE_ACCESS => do_dispatch!(AccessRequest, access),
		#[cfg(feature = "unstable_bmap")]
		Op::FUSE_BMAP => do_dispatch!(BmapRequest, bmap),
		Op::FUSE_COPY_FILE_RANGE => {
			do_dispatch!(CopyFileRangeRequest, copy_file_range)
		},
		Op::FUSE_CREATE => do_dispatch!(CreateRequest, create),
		Op::FUSE_FALLOCATE => do_dispatch!(FallocateRequest, fallocate),
		Op::FUSE_FLUSH => do_dispatch!(FlushRequest, flush),
		Op::FUSE_FORGET | Op::FUSE_BATCH_FORGET => {
			match ForgetRequest::from_fuse_request(&request) {
				Ok(request) => handlers.forget(call, &request),
				Err(err) => {
					#[cfg(feature = "std")]
					if let Some(ref hooks) = hooks {
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
		#[cfg(feature = "unstable_ioctl")]
		Op::FUSE_IOCTL => do_dispatch!(IoctlRequest, ioctl),
		Op::FUSE_LINK => do_dispatch!(LinkRequest, link),
		Op::FUSE_LISTXATTR => do_dispatch!(ListxattrRequest, listxattr),
		Op::FUSE_LOOKUP => do_dispatch!(LookupRequest, lookup),
		Op::FUSE_LSEEK => do_dispatch!(LseekRequest, lseek),
		Op::FUSE_MKDIR => do_dispatch!(MkdirRequest, mkdir),
		Op::FUSE_MKNOD => do_dispatch!(MknodRequest, mknod),
		Op::FUSE_OPEN => do_dispatch!(OpenRequest, open),
		Op::FUSE_OPENDIR => do_dispatch!(OpendirRequest, opendir),
		Op::FUSE_READ => do_dispatch!(ReadRequest, read),
		Op::FUSE_READDIR => do_dispatch!(ReaddirRequest, readdir),
		Op::FUSE_READLINK => do_dispatch!(ReadlinkRequest, readlink),
		Op::FUSE_RELEASE => do_dispatch!(ReleaseRequest, release),
		Op::FUSE_RELEASEDIR => do_dispatch!(ReleasedirRequest, releasedir),
		Op::FUSE_REMOVEXATTR => do_dispatch!(RemovexattrRequest, removexattr),
		Op::FUSE_RENAME | Op::FUSE_RENAME2 => {
			do_dispatch!(RenameRequest, rename)
		},
		Op::FUSE_RMDIR => do_dispatch!(RmdirRequest, rmdir),
		#[cfg(feature = "unstable_setattr")]
		Op::FUSE_SETATTR => do_dispatch!(SetattrRequest, setattr),
		Op::FUSE_SETLK | Op::FUSE_SETLKW => do_dispatch!(SetlkRequest, setlk),
		Op::FUSE_SETXATTR => do_dispatch!(SetxattrRequest, setxattr),
		Op::FUSE_STATFS => do_dispatch!(StatfsRequest, statfs),
		Op::FUSE_SYMLINK => do_dispatch!(SymlinkRequest, symlink),
		Op::FUSE_UNLINK => do_dispatch!(UnlinkRequest, unlink),
		Op::FUSE_WRITE => do_dispatch!(WriteRequest, write),
		_ => {
			#[cfg(feature = "std")]
			if let Some(hooks) = hooks {
				let req = server::UnknownRequest::from_fuse_request(&request);
				hooks.unknown_request(&req);
			}
			let response = ErrorResponse::new(crate::Error::UNIMPLEMENTED);
			response.send(socket, &response_ctx)
		},
	}
}

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
pub trait FuseHandlers<S: FuseSocket> {
	fn access(
		&self,
		call: FuseCall<S>,
		request: &protocol::AccessRequest,
	) -> FuseResult<protocol::AccessResponse, S::Error> {
		call.unimplemented()
	}

	#[cfg(any(doc, feature = "unstable_bmap"))]
	fn bmap(
		&self,
		call: FuseCall<S>,
		request: &protocol::BmapRequest,
	) -> FuseResult<protocol::BmapResponse, S::Error> {
		call.unimplemented()
	}

	fn copy_file_range(
		&self,
		call: FuseCall<S>,
		request: &protocol::CopyFileRangeRequest,
	) -> FuseResult<protocol::CopyFileRangeResponse, S::Error> {
		call.unimplemented()
	}

	fn create(
		&self,
		call: FuseCall<S>,
		request: &protocol::CreateRequest,
	) -> FuseResult<protocol::CreateResponse, S::Error> {
		call.unimplemented()
	}

	fn fallocate(
		&self,
		call: FuseCall<S>,
		request: &protocol::FallocateRequest,
	) -> FuseResult<protocol::FallocateResponse, S::Error> {
		call.unimplemented()
	}

	fn flush(
		&self,
		call: FuseCall<S>,
		request: &protocol::FlushRequest,
	) -> FuseResult<protocol::FlushResponse, S::Error> {
		call.unimplemented()
	}

	fn forget(&self, call: FuseCall<S>, request: &protocol::ForgetRequest) {
		#[cfg(feature = "std")]
		if let Some(hooks) = call.hooks {
			hooks.unhandled_request(call.header);
		}
	}

	fn fsync(
		&self,
		call: FuseCall<S>,
		request: &protocol::FsyncRequest,
	) -> FuseResult<protocol::FsyncResponse, S::Error> {
		call.unimplemented()
	}

	fn fsyncdir(
		&self,
		call: FuseCall<S>,
		request: &protocol::FsyncdirRequest,
	) -> FuseResult<protocol::FsyncdirResponse, S::Error> {
		call.unimplemented()
	}

	fn getattr(
		&self,
		call: FuseCall<S>,
		request: &protocol::GetattrRequest,
	) -> FuseResult<protocol::GetattrResponse, S::Error> {
		call.unimplemented()
	}

	fn getlk(
		&self,
		call: FuseCall<S>,
		request: &protocol::GetlkRequest,
	) -> FuseResult<protocol::GetlkResponse, S::Error> {
		call.unimplemented()
	}

	fn getxattr(
		&self,
		call: FuseCall<S>,
		request: &protocol::GetxattrRequest,
	) -> FuseResult<protocol::GetxattrResponse, S::Error> {
		call.unimplemented()
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	fn ioctl(
		&self,
		call: FuseCall<S>,
		request: &protocol::IoctlRequest,
	) -> FuseResult<protocol::IoctlResponse, S::Error> {
		call.unimplemented()
	}

	fn link(
		&self,
		call: FuseCall<S>,
		request: &protocol::LinkRequest,
	) -> FuseResult<protocol::LinkResponse, S::Error> {
		call.unimplemented()
	}

	fn listxattr(
		&self,
		call: FuseCall<S>,
		request: &protocol::ListxattrRequest,
	) -> FuseResult<protocol::ListxattrResponse, S::Error> {
		call.unimplemented()
	}

	fn lookup(
		&self,
		call: FuseCall<S>,
		request: &protocol::LookupRequest,
	) -> FuseResult<protocol::LookupResponse, S::Error> {
		call.unimplemented()
	}

	fn lseek(
		&self,
		call: FuseCall<S>,
		request: &protocol::LseekRequest,
	) -> FuseResult<protocol::LseekResponse, S::Error> {
		call.unimplemented()
	}

	fn mkdir(
		&self,
		call: FuseCall<S>,
		request: &protocol::MkdirRequest,
	) -> FuseResult<protocol::MkdirResponse, S::Error> {
		call.unimplemented()
	}

	fn mknod(
		&self,
		call: FuseCall<S>,
		request: &protocol::MknodRequest,
	) -> FuseResult<protocol::MknodResponse, S::Error> {
		call.unimplemented()
	}

	fn open(
		&self,
		call: FuseCall<S>,
		request: &protocol::OpenRequest,
	) -> FuseResult<protocol::OpenResponse, S::Error> {
		call.unimplemented()
	}

	fn opendir(
		&self,
		call: FuseCall<S>,
		request: &protocol::OpendirRequest,
	) -> FuseResult<protocol::OpendirResponse, S::Error> {
		call.unimplemented()
	}

	fn read(
		&self,
		call: FuseCall<S>,
		request: &protocol::ReadRequest,
	) -> FuseResult<protocol::ReadResponse, S::Error> {
		call.unimplemented()
	}

	fn readdir(
		&self,
		call: FuseCall<S>,
		request: &protocol::ReaddirRequest,
	) -> FuseResult<protocol::ReaddirResponse, S::Error> {
		call.unimplemented()
	}

	fn readlink(
		&self,
		call: FuseCall<S>,
		request: &protocol::ReadlinkRequest,
	) -> FuseResult<protocol::ReadlinkResponse, S::Error> {
		call.unimplemented()
	}

	fn release(
		&self,
		call: FuseCall<S>,
		request: &protocol::ReleaseRequest,
	) -> FuseResult<protocol::ReleaseResponse, S::Error> {
		call.unimplemented()
	}

	fn releasedir(
		&self,
		call: FuseCall<S>,
		request: &protocol::ReleasedirRequest,
	) -> FuseResult<protocol::ReleasedirResponse, S::Error> {
		call.unimplemented()
	}

	fn removexattr(
		&self,
		call: FuseCall<S>,
		request: &protocol::RemovexattrRequest,
	) -> FuseResult<protocol::RemovexattrResponse, S::Error> {
		call.unimplemented()
	}

	fn rename(
		&self,
		call: FuseCall<S>,
		request: &protocol::RenameRequest,
	) -> FuseResult<protocol::RenameResponse, S::Error> {
		call.unimplemented()
	}

	fn rmdir(
		&self,
		call: FuseCall<S>,
		request: &protocol::RmdirRequest,
	) -> FuseResult<protocol::RmdirResponse, S::Error> {
		call.unimplemented()
	}

	#[cfg(any(doc, feature = "unstable_setattr"))]
	fn setattr(
		&self,
		call: FuseCall<S>,
		request: &protocol::SetattrRequest,
	) -> FuseResult<protocol::SetattrResponse, S::Error> {
		call.unimplemented()
	}

	fn setlk(
		&self,
		call: FuseCall<S>,
		request: &protocol::SetlkRequest,
	) -> FuseResult<protocol::SetlkResponse, S::Error> {
		call.unimplemented()
	}

	fn setxattr(
		&self,
		call: FuseCall<S>,
		request: &protocol::SetxattrRequest,
	) -> FuseResult<protocol::SetxattrResponse, S::Error> {
		call.unimplemented()
	}

	fn statfs(
		&self,
		call: FuseCall<S>,
		request: &protocol::StatfsRequest,
	) -> FuseResult<protocol::StatfsResponse, S::Error> {
		#[cfg(not(target_os = "freebsd"))]
		{
			call.unimplemented()
		}

		#[cfg(target_os = "freebsd")]
		{
			#[cfg(feature = "std")]
			if let Some(hooks) = call.hooks {
				hooks.unhandled_request(call.header);
			}
			let resp = protocol::StatfsResponse::new();
			call.respond_ok(&resp)
		}
	}

	fn symlink(
		&self,
		call: FuseCall<S>,
		request: &protocol::SymlinkRequest,
	) -> FuseResult<protocol::SymlinkResponse, S::Error> {
		call.unimplemented()
	}

	fn unlink(
		&self,
		call: FuseCall<S>,
		request: &protocol::UnlinkRequest,
	) -> FuseResult<protocol::UnlinkResponse, S::Error> {
		call.unimplemented()
	}

	fn write(
		&self,
		call: FuseCall<S>,
		request: &protocol::WriteRequest,
	) -> FuseResult<protocol::WriteResponse, S::Error> {
		call.unimplemented()
	}
}
