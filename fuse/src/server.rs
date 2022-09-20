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

pub mod basic;
mod connection;
mod cuse_connection;
mod cuse_request;
mod fuse_connection;
mod fuse_request;
mod reply;
mod request;

pub use self::cuse_connection::{CuseConnection, CuseConnectionBuilder};
pub use self::cuse_request::{CuseOperation, CuseRequest};
pub use self::fuse_connection::{FuseConnection, FuseConnectionBuilder};
pub use self::fuse_request::{FuseOperation, FuseRequest};
pub use self::reply::{Reply, ReplyInfo};
pub use self::request::RequestHeader;

use crate::io::ProtocolVersion;
use crate::io::{RequestError, ServerRecvError, ServerSendError};
use crate::io::decode::RequestBuf;
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

pub struct CuseRequestBuilder {
	init_flags: CuseInitFlags,
	max_write: u32,
	version: crate::io::ProtocolVersion,
}

impl CuseRequestBuilder {
	pub fn new() -> CuseRequestBuilder {
		CuseRequestBuilder {
			init_flags: CuseInitFlags::new(),
			max_write: DEFAULT_MAX_WRITE,
			version: ProtocolVersion::LATEST,
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

	pub fn version(&mut self, version: ProtocolVersion) -> &mut Self {
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

pub struct FuseRequestBuilder {
	init_flags: FuseInitFlags,
	max_write: u32,
	version: crate::io::ProtocolVersion,
}

impl FuseRequestBuilder {
	pub fn new() -> FuseRequestBuilder {
		FuseRequestBuilder {
			init_flags: FuseInitFlags::new(),
			max_write: DEFAULT_MAX_WRITE,
			version: ProtocolVersion::LATEST,
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

	pub fn version(&mut self, version: ProtocolVersion) -> &mut Self {
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
