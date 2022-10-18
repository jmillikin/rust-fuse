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

//! CUSE and FUSE servers.

use core::cmp;
use core::mem;
use core::num;

use crate::internal::fuse_kernel;
use crate::lock;
use crate::node;
use crate::operations::cuse_init::{
	CuseInitRequest,
	CuseInitResponse,
};
use crate::operations::fuse_init::{
	FuseInitFlag,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::xattr;

pub mod cuse_rpc;
pub mod fuse_rpc;
pub mod io;

pub(crate) mod decode;
pub(crate) mod encode;

pub(crate) mod sealed {
	pub trait Sealed {}
}

// ServerError {{{

/// Errors that may be encountered when serving.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServerError<IoError> {
	/// An invalid request was received from the client.
	RequestError(RequestError),

	/// The socket encountered an I/O error when receiving the next request.
	RecvError(IoError),

	/// The socket encountered an I/O error when sending a response.
	SendError(IoError),
}

impl<E> From<RequestError> for ServerError<E> {
	fn from(err: RequestError) -> Self {
		ServerError::RequestError(err)
	}
}

impl<E> From<io::RecvError<E>> for ServerError<E> {
	fn from(err: io::RecvError<E>) -> Self {
		Self::RecvError(match err {
			io::RecvError::ConnectionClosed(io_err) => io_err,
			io::RecvError::Other(io_err) => io_err,
		})
	}
}

impl<E> From<io::SendError<E>> for ServerError<E> {
	fn from(err: io::SendError<E>) -> Self {
		Self::SendError(match err {
			io::SendError::NotFound(io_err) => io_err,
			io::SendError::Other(io_err) => io_err,
		})
	}
}

// }}}

// RequestError {{{

/// Errors describing why a request is invalid.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestError {
	/// The request contains an invalid [`Lock`].
	///
	/// [`Lock`]: crate::lock::Lock
	LockError(lock::LockError),

	/// The request is missing one or mode node IDs.
	///
	/// For most requests this will mean that the [`RequestHeader::node_id`]
	/// is `None`, but some request types have additional required node IDs
	/// in the request body.
	///
	/// [`RequestHeader::node_id`]: crate::RequestHeader::node_id
	MissingNodeId,

	/// The request is a `FUSE_INTERRUPT` with a missing request ID.
	MissingRequestId,

	/// The request contains an invalid [`node::Name`].
	NodeNameError(node::NameError),

	/// The request contains a timestamp with too many nanoseconds.
	TimestampOverflow,

	/// The request contains an invalid [`xattr::Name`].
	///
	/// [`xattr::Name`]: crate::xattr::Name
	XattrNameError(xattr::NameError),

	/// The request contains an invalid [`xattr::Value`].
	///
	/// [`xattr::Value`]: crate::xattr::Value
	XattrValueError(xattr::ValueError),

	// Errors indicating a programming error in the client.

	/// The request header's request ID is zero.
	InvalidRequestId,

	/// The request buffer contains an incomplete request.
	UnexpectedEof,

	// Errors indicating a programming error in the server.

	/// Attempted to decode a request as the wrong type.
	///
	/// This error indicates a programming error in the server.
	OpcodeMismatch,
}

impl From<lock::LockError> for RequestError {
	fn from(err: lock::LockError) -> RequestError {
		RequestError::LockError(err)
	}
}

impl From<node::NameError> for RequestError {
	fn from(err: node::NameError) -> RequestError {
		RequestError::NodeNameError(err)
	}
}

impl From<xattr::NameError> for RequestError {
	fn from(err: xattr::NameError) -> RequestError {
		RequestError::XattrNameError(err)
	}
}

impl From<xattr::ValueError> for RequestError {
	fn from(err: xattr::ValueError) -> RequestError {
		RequestError::XattrValueError(err)
	}
}

// }}}

// Request {{{

/// A borrowed request received by a CUSE or FUSE server.
///
/// A `Request<'a>` is equivalent to `&'a [u8]`, with additional guarantees
/// around alignment and minimum size.
///
/// The request header can be inspected via [`Request::header`], and conversion
/// into operation-specific request types can be attempted using the
/// [`CuseRequest`] or [`FuseRequest`] traits.
#[derive(Copy, Clone)]
pub struct Request<'a> {
	slice: crate::io::AlignedSlice<'a>,
}

impl<'a> Request<'a> {
	/// Attempts to reborrow an [`AlignedSlice`] as a [`Request`].
	///
	/// # Errors
	///
	/// Returns `UnexpectedEof` if the slice isn't large enough to contain a
	/// [`RequestHeader`], or if the header's request length is larger than
	/// the slice.
	///
	/// Returns `InvalidRequestId` if the header's request ID is zero.
	///
	/// [`AlignedSlice`]: crate::io::AlignedSlice
	/// [`RequestHeader`]: crate::RequestHeader
	#[inline]
	pub fn new(
		slice: crate::io::AlignedSlice<'a>,
	) -> Result<Request<'a>, RequestError> {
		let bytes = slice.get();
		if bytes.len() < mem::size_of::<fuse_kernel::fuse_in_header>() {
			return Err(RequestError::UnexpectedEof);
		}

		let header_ptr = bytes.as_ptr().cast::<fuse_kernel::fuse_in_header>();
		let header = unsafe { &*header_ptr };

		if header.unique == 0 {
			return Err(RequestError::InvalidRequestId);
		}

		let buf_len: u32;
		if mem::size_of::<usize>() > mem::size_of::<u32>() {
			if bytes.len() > u32::MAX as usize {
				buf_len = u32::MAX;
			} else {
				buf_len = bytes.len() as u32;
			}
		} else {
			buf_len = bytes.len() as u32;
		}
		if buf_len < header.len {
			return Err(RequestError::UnexpectedEof);
		}

		Ok(unsafe { Self::new_unchecked(slice) })
	}

	/// Reborrows an [`AlignedSlice`] as a [`Request`], without validation.
	///
	/// # Safety
	///
	/// The slice must be at least large enough to contain a [`RequestHeader`].
	/// The contained header must have a valid length and request ID.
	///
	/// [`AlignedSlice`]: crate::io::AlignedSlice
	/// [`RequestHeader`]: crate::RequestHeader
	#[inline]
	#[must_use]
	pub unsafe fn new_unchecked(
		slice: crate::io::AlignedSlice<'a>,
	) -> Request<'a> {
		Self { slice }
	}

	/// Returns the header of this request.
	#[inline]
	#[must_use]
	pub fn header(self) -> &'a crate::RequestHeader {
		unsafe { &*(self.slice.get().as_ptr().cast()) }
	}

	/// Returns the full contents of this request as a byte slice.
	#[inline]
	#[must_use]
	pub fn as_slice(self) -> &'a [u8] {
		self.slice.get()
	}

	/// Returns the full contents of this request as an aligned byte slice.
	#[inline]
	#[must_use]
	pub fn as_aligned_slice(self) -> crate::io::AlignedSlice<'a> {
		self.slice
	}

	#[inline]
	#[must_use]
	pub(crate) fn decoder(self) -> decode::RequestDecoder<'a> {
		unsafe { decode::RequestDecoder::new_unchecked(self.slice.get()) }
	}
}

// }}}

// CuseRequest {{{

/// Requests that can be received by a CUSE server.
pub trait CuseRequest<'a>: Sized + sealed::Sealed {
	/// Attempt to decode a CUSE server request.
	fn from_request(
		request: Request<'a>,
		request_options: CuseRequestOptions,
	) -> Result<Self, RequestError>;
}

// }}}

// CuseRequestOptions {{{

/// Options for CUSE server request decoding.
#[derive(Clone, Copy)]
pub struct CuseRequestOptions {
	version_minor: u32,
}

impl CuseRequestOptions {
	/// Create a `CuseRequestOptions` from a given CUSE server init response.
	#[inline]
	#[must_use]
	pub fn from_init_response(
		init_response: &CuseInitResponse,
	) -> CuseRequestOptions {
		Self {
			version_minor: init_response.version().minor(),
		}
	}

	#[inline]
	#[must_use]
	pub(crate) fn version_minor(self) -> u32 {
		self.version_minor
	}
}

// }}}

// FuseRequest {{{

/// Requests that can be received by a FUSE server.
pub trait FuseRequest<'a>: Sized + sealed::Sealed {
	/// Attempt to decode a FUSE server request.
	fn from_request(
		request: Request<'a>,
		request_options: FuseRequestOptions,
	) -> Result<Self, RequestError>;
}

// }}}

// FuseRequestOptions {{{

/// Options for FUSE server request decoding.
#[derive(Clone, Copy)]
pub struct FuseRequestOptions {
	version_minor: u16,
	features: u16,
}

const FEATURE_SETXATTR_EXT: u16 = 1 << 0;

impl FuseRequestOptions {
	/// Create a `FuseRequestOptions` from a given FUSE server init response.
	#[inline]
	#[must_use]
	pub fn from_init_response(
		init_response: &FuseInitResponse,
	) -> FuseRequestOptions {
		let mut features = 0;
		if init_response.flags().get(FuseInitFlag::SETXATTR_EXT) {
			features |= FEATURE_SETXATTR_EXT;
		}

		let version = init_response.version();
		let version_minor = if version.minor() > u32::from(u16::MAX) {
			u16::MAX
		} else {
			version.minor() as u16
		};

		Self {
			version_minor,
			features,
		}
	}

	#[inline]
	#[must_use]
	pub(crate) fn version_minor(self) -> u32 {
		u32::from(self.version_minor)
	}

	#[inline]
	#[must_use]
	pub(crate) fn have_setxattr_ext(self) -> bool {
		self.features & FEATURE_SETXATTR_EXT > 0
	}
}

// }}}

// Response {{{

/// A response generated by a CUSE or FUSE server.
pub struct Response<'a> {
	buf: crate::io::SendBuf<'a>,
}

impl<'a> Response<'a> {
	#[inline]
	#[must_use]
	pub(crate) fn new(buf: crate::io::SendBuf<'a>) -> Response<'a> {
		Self { buf }
	}
}

impl<'a> From<Response<'a>> for crate::io::SendBuf<'a> {
	/// Converts a server response into a [`SendBuf`].
	///
	/// [`SendBuf`]: crate::io::SendBuf
	#[inline]
	#[must_use]
	fn from(response: Response<'a>) -> crate::io::SendBuf<'a> {
		response.buf
	}
}

// }}}

// CuseResponse {{{

/// Responses that can be sent by a CUSE server.
pub trait CuseResponse: sealed::Sealed {
	/// Encode a CUSE server response.
	///
	/// The response header will be filled in with the response length and,
	/// if appropriate, its error code.
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		response_options: CuseResponseOptions,
	) -> Response<'a>;
}

// }}}

// CuseResponseOptions {{{

/// Options for CUSE server response encoding.
#[derive(Clone, Copy)]
pub struct CuseResponseOptions {
	_empty: (),
}

impl CuseResponseOptions {
	/// Create a `CuseResponseOptions` from a given FUSE server init response.
	#[inline]
	#[must_use]
	pub fn from_init_response(
		init_response: &CuseInitResponse,
	) -> CuseResponseOptions {
		let _ = init_response;
		Self { _empty: () }
	}
}

// }}}

// FuseResponse {{{

/// Responses that can be sent by a FUSE server.
pub trait FuseResponse: sealed::Sealed {
	/// Encode a FUSE server response.
	///
	/// The response header will be filled in with the response length and,
	/// if appropriate, its error code.
	fn to_response<'a>(
		&'a self,
		response_header: &'a mut crate::ResponseHeader,
		response_options: FuseResponseOptions,
	) -> Response<'a>;
}

/// Options for FUSE server response encoding.
#[derive(Clone, Copy)]
pub struct FuseResponseOptions {
	version_minor: u32,
}

impl FuseResponseOptions {
	/// Create a `FuseResponseOptions` from a given FUSE server init response.
	#[inline]
	#[must_use]
	pub fn from_init_response(
		init_response: &FuseInitResponse,
	) -> FuseResponseOptions {
		Self {
			version_minor: init_response.version().minor(),
		}
	}

	#[inline]
	#[must_use]
	pub(crate) fn version_minor(self) -> u32 {
		self.version_minor
	}
}

// }}}

// Hooks {{{

/// Optional hooks for observing server events.
#[allow(unused_variables)]
pub trait Hooks {
	/// Called for each [`Request`] received by the server.
	fn request(&self, request: Request) {}

	/// Called when decoding a [`Request`] as an operation-specific type fails.
	fn request_error(&self, request: Request, error: RequestError) {}

	/// Called when a [`Request`] is received with an unknown [`Opcode`].
	///
	/// This might happen when the request's `Opcode` isn't recognized by
	/// the library, or when a FUSE-specific request is sent to a CUSE server.
	///
	/// [`Opcode`]: crate::Opcode
	fn unknown_opcode(&self, request: Request) {}

	/// Called when a [`Request`] is received for an unimplemented operation.
	fn unimplemented(&self, request: Request) {}
}

// }}}

/// Perform a CUSE session handshake.
///
/// When a CUSE session is being established the client will send a
/// [`CuseInitRequest`] and the server responds with a [`CuseInitResponse`].
///
/// The response specifies the name and device number of the CUSE device that
/// will be created for this server.
pub fn cuse_init<'a, F, S: io::CuseSocket>(
	socket: &mut S,
	mut init_fn: F,
) -> Result<CuseInitResponse<'a>, ServerError<S::Error>>
where
	F: FnMut(&CuseInitRequest) -> CuseInitResponse<'a>
{
	let mut buf = crate::io::MinReadBuffer::new();

	loop {
		let recv_len = socket.recv(buf.as_slice_mut())?;
		let request = Request::new(buf.as_aligned_slice().truncate(recv_len))?;
		let init_req = CuseInitRequest::from_request(request)?;
		let (response, ok) = cuse_handshake(&init_req, &mut init_fn)?;
		let request_id = request.header().request_id();
		let mut header = crate::ResponseHeader::new(request_id);
		socket.send(response.to_response(&mut header).into())?;
		if ok {
			return Ok(response);
		}
	}
}

pub(crate) fn cuse_handshake<'a, E, F>(
	request: &CuseInitRequest,
	init_fn: &mut F,
) -> Result<(CuseInitResponse<'a>, bool), ServerError<E>>
where
	F: FnMut(&CuseInitRequest) -> CuseInitResponse<'a>
{
	match negotiate_version(request.version()) {
		Some(version) => {
			let mut response = init_fn(request);
			response.set_version(version);
			Ok((response, true))
		},
		None => {
			let mut response = CuseInitResponse::new_nameless();
			response.set_version(crate::Version::LATEST);
			Ok((response, false))
		},
	}
}

/// Perform a FUSE session handshake.
///
/// When a FUSE session is being established the client will send a
/// [`FuseInitRequest`] and the server responds with a [`FuseInitResponse`].
///
/// The response specifies tunable parameters and optional features of the
/// filesystem server.
pub fn fuse_init<F, S: io::FuseSocket>(
	socket: &mut S,
	mut init_fn: F,
) -> Result<FuseInitResponse, ServerError<S::Error>>
where
	F: FnMut(&FuseInitRequest) -> FuseInitResponse,
{
	let mut buf = crate::io::MinReadBuffer::new();

	loop {
		let recv_len = socket.recv(buf.as_slice_mut())?;
		let request = Request::new(buf.as_aligned_slice().truncate(recv_len))?;
		let init_req = FuseInitRequest::from_request(request)?;
		let (response, ok) = fuse_handshake(&init_req, &mut init_fn)?;
		let request_id = request.header().request_id();
		let mut header = crate::ResponseHeader::new(request_id);
		socket.send(response.to_response(&mut header).into())?;
		if ok {
			return Ok(response);
		}
	}
}

pub(crate) fn fuse_handshake<E, F>(
	request: &FuseInitRequest,
	init_fn: &mut F,
) -> Result<(FuseInitResponse, bool), ServerError<E>>
where
	F: FnMut(&FuseInitRequest) -> FuseInitResponse,
{
	match negotiate_version(request.version()) {
		Some(version) => {
			let mut response = init_fn(request);
			response.set_version(version);
			Ok((response, true))
		},
		None => {
			let mut response = FuseInitResponse::new();
			response.set_version(crate::Version::LATEST);
			Ok((response, false))
		},
	}
}

fn negotiate_version(kernel: crate::Version) -> Option<crate::Version> {
	if kernel.major() != crate::Version::LATEST.major() {
		// TODO: hard error on kernel major version < FUSE_KERNEL_VERSION
		return None;
	}
	Some(crate::Version::new(
		crate::Version::LATEST.major(),
		cmp::min(kernel.minor(), crate::Version::LATEST.minor()),
	))
}

/// Receive a CUSE or FUSE request from a [`Socket`].
///
/// Returns `Ok(None)` if the socket's connection has been closed.
///
/// [`Socket`]: io::Socket
pub fn recv<'a, S: io::Socket>(
	socket: &S,
	mut buf: crate::io::AlignedSliceMut<'a>,
) -> Result<Option<Request<'a>>, ServerError<S::Error>> {
	let recv_len = match socket.recv(buf.get_mut()) {
		Ok(len) => len,
		Err(io::RecvError::ConnectionClosed(_)) => return Ok(None),
		Err(err) => return Err(err.into()),
	};
	let recv_buf = buf.truncate(recv_len);
	Ok(Some(Request::new(recv_buf.into())?))
}

/// Send an error response to a [`Socket`].
///
/// [`Socket`]: io::Socket
pub fn send_error<S: io::Socket>(
	socket: &S,
	request_id: num::NonZeroU64,
	error: crate::Error,
) -> Result<(), io::SendError<S::Error>> {
	let mut response_header = crate::ResponseHeader::new(request_id);
	socket.send(encode::error(&mut response_header, error).into())
}
