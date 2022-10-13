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

use core::cell;
use core::cmp;
use core::fmt;
use core::mem;

use crate::internal::fuse_kernel;
use crate::lock;
use crate::node;
use crate::operations::cuse_init::{
	CuseInitFlags,
	CuseInitRequest,
	CuseInitResponse,
};
use crate::operations::fuse_init::{
	FuseInitFlag,
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::xattr;

pub mod cuse_rpc;
pub mod fuse_rpc;
pub mod io;

pub mod decode;
pub(crate) mod encode;

// ServerError {{{

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServerError<IoError> {
	RequestError(RequestError),
	RecvError(IoError),
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

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestError {
	LockError(lock::LockError),
	MissingNodeId,
	MissingRequestId,
	NodeNameError(node::NameError),
	OpcodeMismatch,
	TimestampNanosOverflow,
	UnexpectedEof,
	XattrNameError(xattr::NameError),
	XattrValueError(xattr::ValueError),
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

#[derive(Clone, Copy)]
pub struct ResponseContext {
	pub(crate) request_id: u64,
	pub(crate) version_minor: u32,
}

pub struct CuseRequestBuilder {
	init_flags: CuseInitFlags,
	version: crate::Version,
}

impl CuseRequestBuilder {
	#[must_use]
	pub fn new() -> CuseRequestBuilder {
		CuseRequestBuilder {
			init_flags: CuseInitFlags::new(),
			version: crate::Version::LATEST,
		}
	}

	#[must_use]
	pub fn from_init_response(
		init_response: &CuseInitResponse,
	) -> CuseRequestBuilder {
		CuseRequestBuilder {
			init_flags: init_response.flags(),
			version: init_response.version(),
		}
	}

	pub fn version(&mut self, version: crate::Version) -> &mut Self {
		self.version = version;
		self
	}

	pub fn init_flags(&mut self, init_flags: CuseInitFlags) -> &mut Self {
		self.init_flags = init_flags;
		self
	}

	pub fn build<'a>(
		&self,
		buf: crate::io::AlignedSlice<'a>,
	) -> Result<CuseRequest<'a>, RequestError> {
		Ok(CuseRequest {
			buf: decode::RequestBuf::new(buf)?,
			version_minor: self.version.minor(),
		})
	}
}

pub struct CuseRequest<'a> {
	pub(crate) buf: decode::RequestBuf<'a>,
	pub(crate) version_minor: u32,
}

impl<'a> CuseRequest<'a> {
	pub(crate) fn decoder(&self) -> decode::RequestDecoder<'a> {
		decode::RequestDecoder::new(self.buf)
	}

	#[must_use]
	pub fn header(&self) -> &'a crate::RequestHeader {
		self.buf.header()
	}

	#[must_use]
	pub fn response_context(&self) -> ResponseContext {
		ResponseContext {
			request_id: self.header().request_id().get(),
			version_minor: self.version_minor,
		}
	}
}

pub struct FuseRequestBuilder {
	init_flags: FuseInitFlags,
	version: crate::Version,
}

impl FuseRequestBuilder {
	#[must_use]
	pub fn new() -> FuseRequestBuilder {
		FuseRequestBuilder {
			init_flags: FuseInitFlags::new(),
			version: crate::Version::LATEST,
		}
	}

	#[must_use]
	pub fn from_init_response(
		init_response: &FuseInitResponse,
	) -> FuseRequestBuilder {
		FuseRequestBuilder {
			init_flags: init_response.flags(),
			version: init_response.version(),
		}
	}

	pub fn version(&mut self, version: crate::Version) -> &mut Self {
		self.version = version;
		self
	}

	pub fn init_flags(&mut self, init_flags: FuseInitFlags) -> &mut Self {
		self.init_flags = init_flags;
		self
	}

	pub fn build<'a>(
		&self,
		buf: crate::io::AlignedSlice<'a>,
	) -> Result<FuseRequest<'a>, RequestError> {
		let mut toggles = 0;
		if self.init_flags.get(FuseInitFlag::SETXATTR_EXT) {
			toggles |= TOGGLE_SETXATTR_EXT;
		}
		Ok(FuseRequest {
			buf: decode::RequestBuf::new(buf)?,
			version_minor: self.version.minor(),
			toggles,
		})
	}
}

pub struct FuseRequest<'a> {
	pub(crate) buf: decode::RequestBuf<'a>,
	pub(crate) version_minor: u32,
	toggles: u32,
}

const TOGGLE_SETXATTR_EXT: u32 = 1 << 0;

impl<'a> FuseRequest<'a> {
	pub(crate) fn decoder(&self) -> decode::RequestDecoder<'a> {
		decode::RequestDecoder::new(self.buf)
	}

	#[must_use]
	pub fn header(&self) -> &'a crate::RequestHeader {
		self.buf.header()
	}

	#[must_use]
	pub fn response_context(&self) -> ResponseContext {
		ResponseContext {
			request_id: self.header().request_id().get(),
			version_minor: self.version_minor,
		}
	}

	pub(crate) fn have_setxattr_ext(&self) -> bool {
		self.toggles & TOGGLE_SETXATTR_EXT > 0
	}
}

// UnknownRequest {{{

pub struct UnknownRequest<'a> {
	header: &'a crate::RequestHeader,
	body: cell::RefCell<UnknownBody<'a>>,
}

enum UnknownBody<'a> {
	Raw(decode::RequestBuf<'a>),
	Parsed(Result<&'a [u8], RequestError>),
}

impl<'a> UnknownRequest<'a> {
	#[must_use]
	pub fn from_fuse_request(request: &FuseRequest<'a>) -> Self {
		Self {
			header: request.buf.header(),
			body: cell::RefCell::new(UnknownBody::Raw(request.buf)),
		}
	}

	#[must_use]
	pub fn from_cuse_request(request: &CuseRequest<'a>) -> Self {
		Self {
			header: request.buf.header(),
			body: cell::RefCell::new(UnknownBody::Raw(request.buf)),
		}
	}

	#[must_use]
	pub fn header(&self) -> &crate::RequestHeader {
		self.header
	}

	pub fn body(&self) -> Result<&'a [u8], RequestError> {
		let mut result: Result<&'a [u8], RequestError> = Ok(&[]);
		const HEADER_LEN: usize = mem::size_of::<fuse_kernel::fuse_in_header>();
		self.body.replace_with(|body| match body {
			UnknownBody::Raw(buf) => {
				let body_offset = HEADER_LEN as u32;
				let header = buf.raw_header();
				let mut dec = decode::RequestDecoder::new(*buf);
				result = dec.next_bytes(header.len - body_offset);
				UnknownBody::Parsed(result)
			},
			UnknownBody::Parsed(r) => {
				result = *r;
				UnknownBody::Parsed(*r)
			},
		});
		result
	}
}

impl fmt::Debug for UnknownRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("UnknownRequest")
			.field("header", &self.header())
			.field("body", &format_args!("{:?}", self.body()))
			.finish()
	}
}

// }}}

// ErrorResponse {{{

#[derive(Clone, Copy)]
pub struct ErrorResponse {
	error: crate::Error,
}

impl ErrorResponse {
	#[must_use]
	pub fn new(error: crate::Error) -> ErrorResponse {
		ErrorResponse { error }
	}

	pub fn send<S: io::Socket>(
		&self,
		socket: &S,
		response_ctx: &ResponseContext,
	) -> Result<(), io::SendError<S::Error>> {
		let send = encode::SyncSendOnce::new(socket);
		let enc = encode::ReplyEncoder::new(send, response_ctx.request_id);
		enc.encode_error(self.error)
	}

	pub async fn send_async<S: io::AsyncSocket>(
		&self,
		socket: &S,
		response_ctx: &ResponseContext,
	) -> Result<(), io::SendError<S::Error>> {
		let send = encode::AsyncSendOnce::new(socket);
		let enc = encode::ReplyEncoder::new(send, response_ctx.request_id);
		enc.encode_error(self.error).await
	}
}

// }}}

pub fn cuse_init<'a, S: io::CuseSocket>(
	socket: &mut S,
	mut init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse<'a>,
) -> Result<CuseInitResponse<'a>, ServerError<S::Error>> {
	use crate::server::decode::CuseRequest;

	let mut buf = crate::io::MinReadBuffer::new();

	let req_builder = CuseRequestBuilder::new();
	loop {
		let recv_len = socket.recv(buf.as_slice_mut())?;
		let recv_buf = buf.as_aligned_slice().truncate(recv_len);
		let fuse_req = req_builder.build(recv_buf)?;
		let response_ctx = fuse_req.response_context();
		let init_req = CuseInitRequest::from_cuse_request(&fuse_req)?;
		let (response, ok) = cuse_handshake(&init_req, &mut init_fn)?;
		response.send(socket, &response_ctx)?;
		if ok {
			return Ok(response);
		}
	}
}

pub async fn cuse_init_async<'a, S: io::AsyncCuseSocket>(
	socket: &mut S,
	mut init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse<'a>,
) -> Result<CuseInitResponse<'a>, ServerError<S::Error>> {
	use crate::server::decode::CuseRequest;

	let mut buf = crate::io::MinReadBuffer::new();

	let req_builder = CuseRequestBuilder::new();
	loop {
		let recv_len = socket.recv(buf.as_slice_mut()).await?;
		let recv_buf = buf.as_aligned_slice().truncate(recv_len);
		let fuse_req = req_builder.build(recv_buf)?;
		let response_ctx = fuse_req.response_context();
		let init_req = CuseInitRequest::from_cuse_request(&fuse_req)?;
		let (response, ok) = cuse_handshake(&init_req, &mut init_fn)?;
		response.send_async(socket, &response_ctx).await?;
		if ok {
			return Ok(response);
		}
	}
}

fn cuse_handshake<'a, E>(
	request: &CuseInitRequest,
	init_fn: &mut impl FnMut(&CuseInitRequest) -> CuseInitResponse<'a>,
) -> Result<(CuseInitResponse<'a>, bool), ServerError<E>> {
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

pub fn fuse_init<S: io::FuseSocket>(
	socket: &mut S,
	mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
) -> Result<FuseInitResponse, ServerError<S::Error>> {
	use crate::server::decode::FuseRequest;

	let mut buf = crate::io::MinReadBuffer::new();

	let req_builder = FuseRequestBuilder::new();
	loop {
		let recv_len = socket.recv(buf.as_slice_mut())?;
		let recv_buf = buf.as_aligned_slice().truncate(recv_len);
		let fuse_req = req_builder.build(recv_buf)?;
		let response_ctx = fuse_req.response_context();
		let init_req = FuseInitRequest::from_fuse_request(&fuse_req)?;
		let (response, ok) = fuse_handshake(&init_req, &mut init_fn)?;
		response.send(socket, &response_ctx)?;
		if ok {
			return Ok(response);
		}
	}
}

pub async fn fuse_init_async<S: io::AsyncFuseSocket>(
	socket: &mut S,
	mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
) -> Result<FuseInitResponse, ServerError<S::Error>> {
	use crate::server::decode::FuseRequest;

	let mut buf = crate::io::MinReadBuffer::new();

	let req_builder = FuseRequestBuilder::new();
	loop {
		let recv_len = socket.recv(buf.as_slice_mut()).await?;
		let recv_buf = buf.as_aligned_slice().truncate(recv_len);
		let fuse_req = req_builder.build(recv_buf)?;
		let response_ctx = fuse_req.response_context();
		let init_req = FuseInitRequest::from_fuse_request(&fuse_req)?;
		let (response, ok) = fuse_handshake(&init_req, &mut init_fn)?;
		response.send_async(socket, &response_ctx).await?;
		if ok {
			return Ok(response);
		}
	}
}

fn fuse_handshake<E>(
	request: &FuseInitRequest,
	init_fn: &mut impl FnMut(&FuseInitRequest) -> FuseInitResponse,
) -> Result<(FuseInitResponse, bool), ServerError<E>> {
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

pub struct CuseRequests<'a, S> {
	socket: &'a S,
	builder: CuseRequestBuilder,
}

impl<'a, S> CuseRequests<'a, S> {
	#[must_use]
	pub fn new(
		socket: &'a S,
		init_response: &CuseInitResponse,
	) -> Self {
		Self {
			socket,
			builder: CuseRequestBuilder::from_init_response(init_response),
		}
	}
}

impl<S: io::CuseSocket> CuseRequests<'_, S> {
	pub fn try_next<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Option<CuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = self.socket.recv(buf.get_mut())?;
		let recv_buf = buf.truncate(recv_len);
		Ok(Some(self.builder.build(recv_buf.into())?))
	}
}

pub struct AsyncCuseRequests<'a, S> {
	socket: &'a S,
	builder: CuseRequestBuilder,
}

impl<'a, S> AsyncCuseRequests<'a, S> {
	#[must_use]
	pub fn new(
		socket: &'a S,
		init_response: &CuseInitResponse,
	) -> Self {
		Self {
			socket,
			builder: CuseRequestBuilder::from_init_response(init_response),
		}
	}
}

impl<S: io::AsyncCuseSocket> AsyncCuseRequests<'_, S> {
	pub async fn try_next<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Option<CuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = self.socket.recv(buf.get_mut()).await?;
		let recv_buf = buf.truncate(recv_len);
		Ok(Some(self.builder.build(recv_buf.into())?))
	}
}

pub struct FuseRequests<'a, S> {
	socket: &'a S,
	builder: FuseRequestBuilder,
}

impl<'a, S> FuseRequests<'a, S> {
	#[must_use]
	pub fn new(
		socket: &'a S,
		init_response: &FuseInitResponse,
	) -> Self {
		Self {
			socket,
			builder: FuseRequestBuilder::from_init_response(init_response),
		}
	}
}

impl<S: io::FuseSocket> FuseRequests<'_, S> {
	pub fn try_next<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf.get_mut()) {
			Ok(x) => x,
			Err(io::RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(err) => return Err(err.into()),
		};
		let recv_buf = buf.truncate(recv_len);
		Ok(Some(self.builder.build(recv_buf.into())?))
	}
}

pub struct AsyncFuseRequests<'a, S> {
	socket: &'a S,
	builder: FuseRequestBuilder,
}

impl<'a, S> AsyncFuseRequests<'a, S> {
	#[must_use]
	pub fn new(
		socket: &'a S,
		init_response: &FuseInitResponse,
	) -> Self {
		Self {
			socket,
			builder: FuseRequestBuilder::from_init_response(init_response),
		}
	}
}

impl<S: io::AsyncFuseSocket> AsyncFuseRequests<'_, S> {
	pub async fn try_next<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf.get_mut()).await {
			Ok(x) => x,
			Err(io::RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(err) => return Err(err.into()),
		};
		let recv_buf = buf.truncate(recv_len);
		Ok(Some(self.builder.build(recv_buf.into())?))
	}
}

// Hooks {{{

#[allow(unused_variables)]
pub trait Hooks {
	fn request(&self, header: &crate::RequestHeader) {}

	fn unknown_request(&self, request: &UnknownRequest) {}

	fn unhandled_request(&self, header: &crate::RequestHeader) {}

	fn request_error(&self, header: &crate::RequestHeader, err: RequestError) {}
}

// }}}
