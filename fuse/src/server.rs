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

use crate::io::{RecvError, RequestError, SendError};

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServerError<IoError> {
	RequestError(RequestError),
	ConnectionClosed,
	RequestNotFound,
	RecvError(IoError),
	SendError(IoError),
}

impl<E> From<RequestError> for ServerError<E> {
	fn from(err: RequestError) -> Self {
		ServerError::RequestError(err)
	}
}

impl<E> From<RecvError<E>> for ServerError<E> {
	fn from(err: RecvError<E>) -> Self {
		match err {
			RecvError::ConnectionClosed => Self::ConnectionClosed,
			RecvError::Other(io_err) => Self::RecvError(io_err),
		}
	}
}

impl<E> From<SendError<E>> for ServerError<E> {
	fn from(err: SendError<E>) -> Self {
		match err {
			SendError::NotFound => Self::RequestNotFound,
			SendError::Other(io_err) => Self::RecvError(io_err),
		}
	}
}
