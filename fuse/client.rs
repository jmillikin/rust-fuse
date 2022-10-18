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

//! CUSE and FUSE clients.

pub mod io;

// ClientError {{{

/// Errors that may be encountered by a CUSE or FUSE client.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClientError<IoError> {
	/// An invalid response was received from the server.
	ResponseError(ResponseError),

	/// The socket encountered an I/O error when receiving a response.
	RecvError(IoError),

	/// The socket encountered an I/O error when sending a request.
	SendError(IoError),
}

impl<E> From<ResponseError> for ClientError<E> {
	fn from(err: ResponseError) -> Self {
		Self::ResponseError(err)
	}
}

impl<E> From<io::RecvError<E>> for ClientError<E> {
	fn from(err: io::RecvError<E>) -> Self {
		Self::RecvError(match err {
			io::RecvError::Other(io_err) => io_err,
		})
	}
}

impl<E> From<io::SendError<E>> for ClientError<E> {
	fn from(err: io::SendError<E>) -> Self {
		Self::SendError(match err {
			io::SendError::Other(io_err) => io_err,
		})
	}
}

// }}}

// ResponseError {{{

/// Errors describing why a response is invalid.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResponseError {
}

// }}}
