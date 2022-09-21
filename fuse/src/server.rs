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

use core::cell::RefCell;
use core::fmt;
use core::mem::{size_of, transmute};

pub mod basic;
mod connection;
mod cuse_connection;
mod fuse_connection;
mod reply;

pub use self::cuse_connection::{CuseConnection, CuseConnectionBuilder};
pub use self::fuse_connection::{FuseConnection, FuseConnectionBuilder};
pub use self::reply::Reply;

use crate::Version;
use crate::internal::fuse_kernel::fuse_in_header;
use crate::io::{RequestError, ServerRecvError, ServerSendError};
use crate::io::decode::{RequestDecoder, RequestBuf};
use crate::protocol::cuse_init::{CuseInitFlags, CuseInitResponse};
use crate::protocol::fuse_init::{FuseInitFlags, FuseInitResponse};

const DEFAULT_MAX_WRITE: u32 = 4096;

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

impl<E> From<ServerRecvError<E>> for ServerError<E> {
	fn from(err: ServerRecvError<E>) -> Self {
		Self::RecvError(match err {
			ServerRecvError::ConnectionClosed(io_err) => io_err,
			ServerRecvError::Other(io_err) => io_err,
		})
	}
}

impl<E> From<ServerSendError<E>> for ServerError<E> {
	fn from(err: ServerSendError<E>) -> Self {
		Self::RecvError(match err {
			ServerSendError::NotFound(io_err) => io_err,
			ServerSendError::Other(io_err) => io_err,
		})
	}
}

#[derive(Copy, Clone)]
pub struct RequestHeader {
	raw: fuse_in_header,
}

impl RequestHeader {
	pub(crate) fn new_ref<'a>(raw: &'a fuse_in_header) -> &'a RequestHeader {
		unsafe { transmute(raw) }
	}

	pub const fn opcode(&self) -> crate::Opcode {
		crate::Opcode(self.raw.opcode.0)
	}

	pub const fn request_id(&self) -> u64 {
		self.raw.unique
	}

	pub fn node_id(&self) -> Option<crate::NodeId> {
		crate::NodeId::new(self.raw.nodeid)
	}

	pub const fn user_id(&self) -> u32 {
		self.raw.uid
	}

	pub const fn group_id(&self) -> u32 {
		self.raw.gid
	}

	pub const fn process_id(&self) -> u32 {
		self.raw.pid
	}

	pub const fn len(&self) -> u32 {
		self.raw.len
	}
}

impl fmt::Debug for RequestHeader {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RequestHeader")
			.field("opcode", &self.raw.opcode)
			.field("request_id", &self.raw.unique)
			.field("node_id", &format_args!("{:?}", self.node_id()))
			.field("user_id", &self.raw.uid)
			.field("group_id", &self.raw.gid)
			.field("process_id", &self.raw.pid)
			.field("len", &self.raw.len)
			.finish()
	}
}

#[derive(Clone, Copy)]
pub struct ResponseContext {
	pub(crate) request_id: u64,
	pub(crate) version_minor: u32,
}

pub struct CuseRequestBuilder {
	init_flags: CuseInitFlags,
	max_write: u32,
	version: Version,
}

impl CuseRequestBuilder {
	pub fn new() -> CuseRequestBuilder {
		CuseRequestBuilder {
			init_flags: CuseInitFlags::new(),
			max_write: DEFAULT_MAX_WRITE,
			version: Version::LATEST,
		}
	}

	pub fn from_init_response(
		init_response: &CuseInitResponse,
	) -> CuseRequestBuilder {
		CuseRequestBuilder {
			init_flags: *init_response.flags(),
			max_write: init_response.max_write(),
			version: init_response.version(),
		}
	}

	pub fn version(&mut self, version: Version) -> &mut Self {
		self.version = version;
		self
	}

	pub fn init_flags(&mut self, init_flags: CuseInitFlags) -> &mut Self {
		self.init_flags = init_flags;
		self
	}

	pub fn max_write(&mut self, max_write: u32) -> &mut Self {
		self.max_write = max_write;
		self
	}

	pub fn build<'a>(&self, buf: &'a [u8]) -> Result<CuseRequest<'a>, RequestError> {
		Ok(CuseRequest {
			buf: RequestBuf::new(buf)?,
			version_minor: self.version.minor(),
		})
	}
}

pub struct CuseRequest<'a> {
	pub(crate) buf: RequestBuf<'a>,
	pub(crate) version_minor: u32,
}

impl<'a> CuseRequest<'a> {
	pub(crate) fn decoder(&self) -> RequestDecoder<'a> {
		RequestDecoder::new(self.buf)
	}

	pub fn header(&self) -> &'a RequestHeader {
		RequestHeader::new_ref(self.buf.header())
	}

	pub fn response_context(&self) -> ResponseContext {
		ResponseContext {
			request_id: self.header().request_id(),
			version_minor: self.version_minor,
		}
	}
}

pub struct FuseRequestBuilder {
	init_flags: FuseInitFlags,
	max_write: u32,
	version: Version,
}

impl FuseRequestBuilder {
	pub fn new() -> FuseRequestBuilder {
		FuseRequestBuilder {
			init_flags: FuseInitFlags::new(),
			max_write: DEFAULT_MAX_WRITE,
			version: Version::LATEST,
		}
	}

	pub fn from_init_response(
		init_response: &FuseInitResponse,
	) -> FuseRequestBuilder {
		FuseRequestBuilder {
			init_flags: *init_response.flags(),
			max_write: init_response.max_write(),
			version: init_response.version(),
		}
	}

	pub fn version(&mut self, version: Version) -> &mut Self {
		self.version = version;
		self
	}

	pub fn init_flags(&mut self, init_flags: FuseInitFlags) -> &mut Self {
		self.init_flags = init_flags;
		self
	}

	pub fn max_write(&mut self, max_write: u32) -> &mut Self {
		self.max_write = max_write;
		self
	}

	pub fn build<'a>(&self, buf: &'a [u8]) -> Result<FuseRequest<'a>, RequestError> {
		Ok(FuseRequest {
			buf: RequestBuf::new(buf)?,
			version_minor: self.version.minor(),
		})
	}
}

pub struct FuseRequest<'a> {
	pub(crate) buf: RequestBuf<'a>,
	pub(crate) version_minor: u32,
}

impl<'a> FuseRequest<'a> {
	pub(crate) fn decoder(&self) -> RequestDecoder<'a> {
		RequestDecoder::new(self.buf)
	}

	pub fn header(&self) -> &'a RequestHeader {
		RequestHeader::new_ref(self.buf.header())
	}

	pub fn response_context(&self) -> ResponseContext {
		ResponseContext {
			request_id: self.header().request_id(),
			version_minor: self.version_minor,
		}
	}
}

pub struct UnknownRequest<'a> {
	header: &'a fuse_in_header,
	body: RefCell<UnknownBody<'a>>,
}

enum UnknownBody<'a> {
	Raw(RequestBuf<'a>),
	Parsed(Result<&'a [u8], RequestError>),
}

impl<'a> UnknownRequest<'a> {
	pub fn from_fuse_request(request: &FuseRequest<'a>) -> Self {
		Self {
			header: request.buf.header(),
			body: RefCell::new(UnknownBody::Raw(request.buf)),
		}
	}

	pub fn from_cuse_request(request: &CuseRequest<'a>) -> Self {
		Self {
			header: request.buf.header(),
			body: RefCell::new(UnknownBody::Raw(request.buf)),
		}
	}

	pub fn header(&self) -> &RequestHeader {
		RequestHeader::new_ref(&self.header)
	}

	pub fn body(&self) -> Result<&'a [u8], RequestError> {
		let mut result: Result<&'a [u8], RequestError> = Ok(&[]);
		self.body.replace_with(|body| match body {
			UnknownBody::Raw(buf) => {
				let body_offset = size_of::<fuse_in_header>() as u32;
				let mut dec = RequestDecoder::new(*buf);
				result = dec.next_bytes(self.header.len - body_offset);
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
