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
use crate::io::decode::RequestBuf;
use crate::io::encode::{AsyncSendOnce, ReplyEncoder, SyncSendOnce};
use crate::protocol::fuse_init::{
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server::{FuseRequest, FuseRequestBuilder, Reply, ReplyInfo, ServerError};
use crate::server::connection::negotiate_version;

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
		socket: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_reply = Self::handshake(&socket, init_fn)?;
		Ok(Self {
			socket,
			version: init_reply.version(),
		})
	}

	fn handshake(
		socket: &S,
		mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<FuseInitResponse, ServerError<S::Error>> {
		let mut buf = io::ArrayBuffer::new();
		let buf = buf.borrow_mut();

		loop {
			let recv = socket.recv(buf)?;
			let (reply, request_id, ok) = handshake(&buf, recv, &mut init_fn)?;

			reply.encode(SyncSendOnce::new(socket), request_id)?;

			if ok {
				return Ok(reply);
			}
		}
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

	pub fn reply_ok(
		&self,
		request_id: u64,
		reply: &impl Reply,
	) -> Result<(), io::SendError<S::Error>> {
		reply.send(
			&self.socket,
			ReplyInfo {
				request_id,
				version_minor: self.version.minor(),
			},
		)?;
		Ok(())
	}

	pub fn reply_err(
		&self,
		request_id: u64,
		error: crate::Error,
	) -> Result<(), io::SendError<S::Error>> {
		let send = SyncSendOnce::new(&self.socket);
		let enc = ReplyEncoder::new(send, request_id);
		enc.encode_error(error)?;
		Ok(())
	}
}

pub struct AsyncFuseConnection<S> {
	socket: S,
	version: Version,
}

impl<S: io::AsyncFuseServerSocket> AsyncFuseConnection<S> {
	pub async fn new(
		socket: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_reply = Self::handshake(&socket, init_fn).await?;
		Ok(Self {
			socket,
			version: init_reply.version(),
		})
	}

	async fn handshake(
		socket: &S,
		mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<FuseInitResponse, ServerError<S::Error>> {
		let mut buf = io::ArrayBuffer::new();
		let buf = buf.borrow_mut();

		loop {
			let recv = socket.recv(buf).await?;
			let (reply, req_id, done) = handshake(&buf, recv, &mut init_fn)?;

			reply.encode(AsyncSendOnce::new(socket), req_id).await?;

			if done {
				return Ok(reply);
			}
		}
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

	pub async fn reply_ok(
		&self,
		request_id: u64,
		reply: &impl Reply,
	) -> Result<(), io::SendError<S::Error>> {
		reply.send_async(
			&self.socket,
			ReplyInfo {
				request_id,
				version_minor: self.version.minor(),
			},
		).await?;
		Ok(())
	}

	pub async fn reply_err(
		&self,
		request_id: u64,
		error: crate::Error,
	) -> Result<(), io::SendError<S::Error>> {
		let send = AsyncSendOnce::new(&self.socket);
		let enc = ReplyEncoder::new(send, request_id);
		enc.encode_error(error).await?;
		Ok(())
	}
}

fn handshake<E>(
	recv_buf: &[u8],
	recv_len: usize,
	init_fn: &mut impl FnMut(&FuseInitRequest) -> FuseInitResponse,
) -> Result<(FuseInitResponse, u64, bool), ServerError<E>> {
	let v_latest = Version::LATEST;
	let v_minor = v_latest.minor();

	let request_buf = RequestBuf::new(&recv_buf[..recv_len])?;
	let init_request = FuseInitRequest::from_fuse_request(&FuseRequest {
		buf: request_buf,
		version_minor: v_minor,
	})?;

	let mut done = false;
	let init_reply = match negotiate_version(init_request.version()) {
		Some(version) => {
			let mut init_reply = init_fn(&init_request);
			init_reply.set_version(version);
			done = true;
			init_reply
		},
		None => {
			let mut init_reply = FuseInitResponse::new();
			init_reply.set_version(v_latest);
			init_reply
		},
	};

	let request_id = request_buf.header().unique;
	Ok((init_reply, request_id, done))
}
