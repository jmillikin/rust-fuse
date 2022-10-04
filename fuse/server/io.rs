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

use crate::io::SendBuf;
use crate::xattr;

pub(crate) mod decode;
pub(crate) mod encode;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestError {
	InvalidLockType,
	MissingNodeId,
	OpcodeMismatch,
	UnexpectedEof,
	XattrNameError(xattr::NameError),
	XattrValueError(xattr::ValueError),
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

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecvError<IoError> {
	ConnectionClosed(IoError),
	Other(IoError),
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SendError<IoError> {
	NotFound(IoError),
	Other(IoError),
}

pub trait Socket {
	type Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Self::Error>>;

	fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>>;
}

impl<S: Socket> Socket for &S {
	type Error = S::Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Self::Error>> {
		(*self).recv(buf)
	}

	fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>> {
		(*self).send(buf)
	}
}

pub trait CuseSocket: Socket {}

impl<S: CuseSocket> CuseSocket for &S {}

pub trait FuseSocket: Socket {}

impl<S: FuseSocket> FuseSocket for &S {}

pub trait AsyncSocket {
	type Error;
	type RecvFuture: Future<Output = Result<usize, RecvError<Self::Error>>>;
	type SendFuture: Future<Output = Result<(), SendError<Self::Error>>>;

	fn recv(&self, buf: &mut [u8]) -> Self::RecvFuture;

	fn send(&self, buf: SendBuf) -> Self::SendFuture;
}

impl<S: AsyncSocket> AsyncSocket for &S {
	type Error = S::Error;
	type RecvFuture = S::RecvFuture;
	type SendFuture = S::SendFuture;

	fn recv(&self, buf: &mut [u8]) -> Self::RecvFuture {
		(*self).recv(buf)
	}

	fn send(&self, buf: SendBuf) -> Self::SendFuture {
		(*self).send(buf)
	}
}

pub trait AsyncCuseSocket: AsyncSocket {}

impl<S: AsyncCuseSocket> AsyncCuseSocket for &S {}

pub trait AsyncFuseSocket: AsyncSocket {}

impl<S: AsyncFuseSocket> AsyncFuseSocket for &S {}

#[cfg(feature = "std")]
mod std_impls {
	use std::rc::Rc;
	use std::sync::Arc;

	use super::*;

	impl<S: Socket> Socket for Arc<S> {
		type Error = S::Error;

		fn recv(
			&self,
			buf: &mut [u8],
		) -> Result<usize, RecvError<Self::Error>> {
			Arc::as_ref(self).recv(buf)
		}

		fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>> {
			Arc::as_ref(self).send(buf)
		}
	}

	impl<S: AsyncSocket> AsyncSocket for Arc<S> {
		type Error = S::Error;
		type RecvFuture = S::RecvFuture;
		type SendFuture = S::SendFuture;

		fn recv(&self, buf: &mut [u8]) -> Self::RecvFuture {
			Arc::as_ref(self).recv(buf)
		}

		fn send(&self, buf: SendBuf) -> Self::SendFuture {
			Arc::as_ref(self).send(buf)
		}
	}

	impl<S: CuseSocket> CuseSocket for Arc<S> {}
	impl<S: FuseSocket> FuseSocket for Arc<S> {}
	impl<S: AsyncCuseSocket> AsyncCuseSocket for Arc<S> {}
	impl<S: AsyncFuseSocket> AsyncFuseSocket for Arc<S> {}

	impl<S: Socket> Socket for Box<S> {
		type Error = S::Error;

		fn recv(
			&self,
			buf: &mut [u8],
		) -> Result<usize, RecvError<Self::Error>> {
			Box::as_ref(self).recv(buf)
		}

		fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>> {
			Box::as_ref(self).send(buf)
		}
	}

	impl<S: AsyncSocket> AsyncSocket for Box<S> {
		type Error = S::Error;
		type RecvFuture = S::RecvFuture;
		type SendFuture = S::SendFuture;

		fn recv(&self, buf: &mut [u8]) -> Self::RecvFuture {
			Box::as_ref(self).recv(buf)
		}

		fn send(&self, buf: SendBuf) -> Self::SendFuture {
			Box::as_ref(self).send(buf)
		}
	}

	impl<S: CuseSocket> CuseSocket for Box<S> {}
	impl<S: FuseSocket> FuseSocket for Box<S> {}
	impl<S: AsyncCuseSocket> AsyncCuseSocket for Box<S> {}
	impl<S: AsyncFuseSocket> AsyncFuseSocket for Box<S> {}

	impl<S: Socket> Socket for Rc<S> {
		type Error = S::Error;

		fn recv(
			&self,
			buf: &mut [u8],
		) -> Result<usize, RecvError<Self::Error>> {
			Rc::as_ref(self).recv(buf)
		}

		fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>> {
			Rc::as_ref(self).send(buf)
		}
	}

	impl<S: AsyncSocket> AsyncSocket for Rc<S> {
		type Error = S::Error;
		type RecvFuture = S::RecvFuture;
		type SendFuture = S::SendFuture;

		fn recv(&self, buf: &mut [u8]) -> Self::RecvFuture {
			Rc::as_ref(self).recv(buf)
		}

		fn send(&self, buf: SendBuf) -> Self::SendFuture {
			Rc::as_ref(self).send(buf)
		}
	}

	impl<S: CuseSocket> CuseSocket for Rc<S> {}
	impl<S: FuseSocket> FuseSocket for Rc<S> {}
	impl<S: AsyncCuseSocket> AsyncCuseSocket for Rc<S> {}
	impl<S: AsyncFuseSocket> AsyncFuseSocket for Rc<S> {}
}
