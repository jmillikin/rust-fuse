// Copyright 2022 John Millikin and the rust-fuse contributors.
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

//! Server-specific async I/O types.

use crate::io::SendBuf;

pub use crate::server::io::{
	RecvError,
	SendError,
};

/// Trait for async sockets that can receive requests and send responses.
pub trait Socket {
	/// Type of errors that may be returned from this socket's I/O methods.
	type Error;

	/// Receive a single serialised request from the client.
	///
	/// The buffer must be large enough to contain any request that might be
	/// received for the current session's negotiated maximum message size.
	async fn recv(
		&self,
		buf: &mut [u8],
	) -> Result<usize, RecvError<Self::Error>>;

	/// Send a single serialised response to the client.
	async fn send(
		&self,
		buf: SendBuf<'_>,
	) -> Result<(), SendError<Self::Error>>;
}

/// Marker trait for async CUSE sockets.
pub trait CuseSocket: Socket {}

/// Marker trait for async FUSE sockets.
pub trait FuseSocket: Socket {}

impl<S: Socket> Socket for &S {
	type Error = S::Error;

	async fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<S::Error>> {
		(*self).recv(buf).await
	}

	async fn send(&self, buf: SendBuf<'_>) -> Result<(), SendError<S::Error>> {
		(*self).send(buf).await
	}
}

impl<S: CuseSocket> CuseSocket for &S {}

impl<S: FuseSocket> FuseSocket for &S {}

#[cfg(feature = "alloc")]
mod std_impls {
	use alloc::boxed::Box;
	use alloc::rc::Rc;
	use alloc::sync::Arc;

	use super::*;

	impl<S: Socket> Socket for Arc<S> {
		type Error = S::Error;

		async fn recv(
			&self,
			buf: &mut [u8],
		) -> Result<usize, RecvError<S::Error>> {
			Arc::as_ref(self).recv(buf).await
		}

		async fn send(
			&self,
			buf: SendBuf<'_>,
		) -> Result<(), SendError<S::Error>> {
			Arc::as_ref(self).send(buf).await
		}
	}

	impl<S: CuseSocket> CuseSocket for Arc<S> {}
	impl<S: FuseSocket> FuseSocket for Arc<S> {}

	impl<S: Socket> Socket for Box<S> {
		type Error = S::Error;

		async fn recv(
			&self,
			buf: &mut [u8],
		) -> Result<usize, RecvError<S::Error>> {
			Box::as_ref(self).recv(buf).await
		}

		async fn send(
			&self,
			buf: SendBuf<'_>,
		) -> Result<(), SendError<S::Error>> {
			Box::as_ref(self).send(buf).await
		}
	}

	impl<S: CuseSocket> CuseSocket for Box<S> {}
	impl<S: FuseSocket> FuseSocket for Box<S> {}

	impl<S: Socket> Socket for Rc<S> {
		type Error = S::Error;

		async fn recv(
			&self,
			buf: &mut [u8],
		) -> Result<usize, RecvError<S::Error>> {
			Rc::as_ref(self).recv(buf).await
		}

		async fn send(
			&self,
			buf: SendBuf<'_>,
		) -> Result<(), SendError<S::Error>> {
			Rc::as_ref(self).send(buf).await
		}
	}

	impl<S: CuseSocket> CuseSocket for Rc<S> {}
	impl<S: FuseSocket> FuseSocket for Rc<S> {}
}
