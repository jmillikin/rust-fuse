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

//! Server-specific I/O types.

use crate::io::SendBuf;

/// Errors that may be encountered when receiving a request.
///
/// Sockets may use the variants of this enum to provide hints to server code
/// about the nature and severity of errors.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecvError<IoError> {
	/// The connection has been cleanly closed by the client.
	ConnectionClosed(IoError),

	/// The socket encountered an error not otherwise specified.
	Other(IoError),
}

/// Errors that may be encountered when sending a response.
///
/// Sockets may use the variants of this enum to provide hints to server code
/// about the nature and severity of errors.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SendError<IoError> {
	/// The response's original request has been forgotten by the client.
	///
	/// The server should treat this as a non-fatal error.
	NotFound(IoError),

	/// The socket encountered an error not otherwise specified.
	Other(IoError),
}

/// Trait for sockets that can receive requests and send responses.
pub trait Socket {
	/// Type of errors that may be returned from this socket's I/O methods.
	type Error;

	/// Receive a single serialised request from the client.
	///
	/// The buffer must be large enough to contain any request that might be
	/// received for the current session's negotiated maximum message size.
	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Self::Error>>;

	/// Send a single serialised response to the client.
	fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>>;
}

/// Marker trait for CUSE sockets.
pub trait CuseSocket: Socket {}

/// Marker trait for FUSE sockets.
pub trait FuseSocket: Socket {}

impl<S: Socket> Socket for &S {
	type Error = S::Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<S::Error>> {
		(*self).recv(buf)
	}

	fn send(&self, buf: SendBuf) -> Result<(), SendError<S::Error>> {
		(*self).send(buf)
	}
}

impl<S: CuseSocket> CuseSocket for &S {}

impl<S: FuseSocket> FuseSocket for &S {}
