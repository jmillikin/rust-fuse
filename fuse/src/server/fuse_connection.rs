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

use crate::Version;
use crate::io::{self, ServerRecvError as RecvError};
use crate::protocol::fuse_init::{
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server::{FuseRequest, FuseRequestBuilder, ServerError};

pub struct FuseConnectionBuilder<S> {
	socket: S,
	flags: FuseInitFlags,
}

impl<S> FuseConnectionBuilder<S> {
	pub fn new(socket: S) -> Self {
		Self {
			socket,
			flags: FuseInitFlags::new(),
		}
	}
}

impl<S: io::FuseServerSocket> FuseConnectionBuilder<S> {
	pub fn build(self) -> Result<FuseConnection<S>, ServerError<S::Error>> {
		let flags = self.flags;
		FuseConnection::new(self.socket, |_request| {
			let mut reply = FuseInitResponse::new();
			*reply.flags_mut() = flags;
			reply
		})
	}
}

impl<S: io::AsyncFuseServerSocket> FuseConnectionBuilder<S> {
	pub async fn build_async(
		self,
	) -> Result<AsyncFuseConnection<S>, ServerError<S::Error>> {
		let flags = self.flags;
		AsyncFuseConnection::new(self.socket, |_request| {
			let mut reply = FuseInitResponse::new();
			*reply.flags_mut() = flags;
			reply
		}).await
	}
}

pub struct FuseConnection<S> {
	socket: S,
	version: Version,
}

impl<S: io::FuseServerSocket> FuseConnection<S> {
	pub fn new(
		mut socket: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_reply = super::fuse_init(&mut socket, init_fn)?;
		Ok(Self {
			socket,
			version: init_reply.version(),
		})
	}

	pub(crate) fn socket(&self) -> &S {
		&self.socket
	}
}

impl<S> FuseConnection<S> {
	pub fn version(&self) -> Version {
		self.version
	}

	pub fn try_clone<Error>(
		&self,
		clone_socket: impl FnOnce(&S) -> Result<S, Error>,
	) -> Result<Self, Error> {
		Ok(Self {
			socket: clone_socket(&self.socket)?,
			version: self.version,
		})
	}
}

impl<S: io::FuseServerSocket> FuseConnection<S> {
	pub fn recv<'a>(
		&self,
		buf: &'a mut [u8],
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf) {
			Ok(x) => x,
			Err(RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(RecvError::Other(err)) => return Err(ServerError::RecvError(err)),
		};
		let request = FuseRequestBuilder::new()
			.version(self.version)
			.build(&buf[..recv_len])?;
		Ok(Some(request))
	}
}

pub struct AsyncFuseConnection<S> {
	socket: S,
	version: Version,
}

impl<S: io::AsyncFuseServerSocket> AsyncFuseConnection<S> {
	pub async fn new(
		mut socket: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_reply = super::fuse_init_async(&mut socket, init_fn).await?;
		Ok(Self {
			socket,
			version: init_reply.version(),
		})
	}
}

impl<S> AsyncFuseConnection<S> {
	pub fn version(&self) -> Version {
		self.version
	}

	pub fn try_clone<Error>(
		&self,
		clone_socket: impl FnOnce(&S) -> Result<S, Error>,
	) -> Result<Self, Error> {
		Ok(Self {
			socket: clone_socket(&self.socket)?,
			version: self.version,
		})
	}
}

impl<S: io::AsyncFuseServerSocket> AsyncFuseConnection<S> {
	pub async fn recv<'a>(
		&self,
		buf: &'a mut [u8],
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf).await {
			Ok(x) => x,
			Err(RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(RecvError::Other(err)) => return Err(ServerError::RecvError(err)),
		};
		let request = FuseRequestBuilder::new()
			.version(self.version)
			.build(&buf[..recv_len])?;
		Ok(Some(request))
	}
}
