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

use core::future::Future;

mod buffer;
pub(crate) mod decode;
pub(crate) mod encode;
mod version;

pub use self::buffer::ArrayBuffer;
pub use self::decode::{ReplyError, RequestError};

pub use self::version::ProtocolVersion;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServerRecvError<IoError> {
	ConnectionClosed(IoError),
	Other(IoError),
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServerSendError<IoError> {
	NotFound(IoError),
	Other(IoError),
}

pub type SendError<E> = ServerSendError<E>;

pub trait ServerSocket {
	type Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, ServerRecvError<Self::Error>>;

	fn send(&self, buf: &[u8]) -> Result<(), ServerSendError<Self::Error>>;

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), ServerSendError<Self::Error>>;
}

pub trait CuseServerSocket: ServerSocket {}

pub trait FuseServerSocket: ServerSocket {}

pub trait AsyncServerSocket {
	type Error;
	type RecvFuture: Future<Output = Result<usize, ServerRecvError<Self::Error>>>;
	type SendFuture: Future<Output = Result<(), ServerSendError<Self::Error>>>;

	fn recv(&self, buf: &mut [u8]) -> Self::RecvFuture;

	fn send(&self, buf: &[u8]) -> Self::SendFuture;

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Self::SendFuture;
}

pub trait AsyncCuseServerSocket: AsyncServerSocket {}

pub trait AsyncFuseServerSocket: AsyncServerSocket {}
