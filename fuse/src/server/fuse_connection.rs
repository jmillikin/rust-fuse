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

use crate::io::{self, Buffer, RecvError};
use crate::io::decode::RequestBuf;
use crate::io::encode::{AsyncSendOnce, ReplyEncoder, SyncSendOnce};
use crate::protocol::fuse_init::{
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server::{FuseRequest, Reply, ReplyInfo, ServerError};
use crate::server::connection::negotiate_version;

pub struct FuseConnectionBuilder<Stream> {
	stream: Stream,
	flags: FuseInitFlags,
}

impl<'a, Stream> FuseConnectionBuilder<Stream> {
	pub fn new(stream: Stream) -> Self {
		Self {
			stream,
			flags: FuseInitFlags::new(),
		}
	}
}

impl<S> FuseConnectionBuilder<S> {
}

impl<S, E> FuseConnectionBuilder<S>
where
	S: io::InputStream<Error = E> + io::OutputStream<Error = E>,
{
	pub fn build(self) -> Result<FuseConnection<S>, ServerError<E>> {
		let flags = self.flags;
		FuseConnection::new(self.stream, |_request| {
			let mut reply = FuseInitResponse::new();
			*reply.flags_mut() = flags;
			reply
		})
	}
}

impl<S, E> FuseConnectionBuilder<S>
where
	S: io::AsyncInputStream<Error = E> + io::AsyncOutputStream<Error = E>,
{
	pub async fn build_async(
		self,
	) -> Result<AsyncFuseConnection<S>, ServerError<E>> {
		let flags = self.flags;
		AsyncFuseConnection::new(self.stream, |_request| {
			let mut reply = FuseInitResponse::new();
			*reply.flags_mut() = flags;
			reply
		}).await
	}
}

pub struct FuseConnection<Stream> {
	stream: Stream,
	version: io::ProtocolVersion,
}

impl<S, E> FuseConnection<S>
where
	S: io::InputStream<Error = E> + io::OutputStream<Error = E>,
{
	pub fn new(
		stream: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<E>> {
		let init_reply = Self::handshake(&stream, init_fn)?;
		Ok(Self {
			stream,
			version: init_reply.version(),
		})
	}

	fn handshake(
		stream: &S,
		mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<FuseInitResponse, ServerError<E>> {
		let mut buf = io::ArrayBuffer::new();

		loop {
			let recv = stream.recv(buf.borrow_mut())?;
			let (reply, request_id, ok) = handshake(&buf, recv, &mut init_fn)?;

			reply.encode(SyncSendOnce::new(stream), request_id)?;

			if ok {
				return Ok(reply);
			}
		}
	}
}

impl<S> FuseConnection<S> {
	pub fn version(&self) -> io::ProtocolVersion {
		self.version
	}

	pub fn try_clone<Error>(
		&self,
		clone_stream: impl FnOnce(&S) -> Result<S, Error>,
	) -> Result<Self, Error> {
		Ok(Self {
			stream: clone_stream(&self.stream)?,
			version: self.version,
		})
	}
}

impl<S: io::InputStream> FuseConnection<S> {
	pub fn recv<'a>(
		&self,
		buf: &'a mut impl io::Buffer,
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.stream.recv(buf.borrow_mut()) {
			Ok(x) => x,
			Err(RecvError::ConnectionClosed) => return Ok(None),
			Err(RecvError::Other(err)) => return Err(ServerError::RecvError(err)),
		};
		let v_minor = self.version.minor();
		Ok(Some(FuseRequest::new(buf, recv_len, v_minor)?))
	}
}

impl<S: io::OutputStream> FuseConnection<S> {
	pub fn reply_ok(
		&self,
		request_id: u64,
		reply: &impl Reply,
	) -> Result<(), io::SendError<S::Error>> {
		reply.send(
			&self.stream,
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
		let send = SyncSendOnce::new(&self.stream);
		let enc = ReplyEncoder::new(send, request_id);
		enc.encode_error(error)?;
		Ok(())
	}
}

pub struct AsyncFuseConnection<Stream> {
	stream: Stream,
	version: io::ProtocolVersion,
}

impl<S, E> AsyncFuseConnection<S>
where
	S: io::AsyncInputStream<Error = E> + io::AsyncOutputStream<Error = E>,
{
	pub async fn new(
		stream: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<E>> {
		let init_reply = Self::handshake(&stream, init_fn).await?;
		Ok(Self {
			stream,
			version: init_reply.version(),
		})
	}

	async fn handshake(
		stream: &S,
		mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<FuseInitResponse, ServerError<E>> {
		let mut buf = io::ArrayBuffer::new();

		loop {
			let recv = stream.recv(buf.borrow_mut()).await?;
			let (reply, req_id, done) = handshake(&buf, recv, &mut init_fn)?;

			reply.encode(AsyncSendOnce::new(stream), req_id).await?;

			if done {
				return Ok(reply);
			}
		}
	}
}

impl<S> AsyncFuseConnection<S> {
	pub fn version(&self) -> io::ProtocolVersion {
		self.version
	}

	pub fn try_clone<Error>(
		&self,
		clone_stream: impl FnOnce(&S) -> Result<S, Error>,
	) -> Result<Self, Error> {
		Ok(Self {
			stream: clone_stream(&self.stream)?,
			version: self.version,
		})
	}
}

impl<S: io::AsyncInputStream> AsyncFuseConnection<S> {
	pub async fn recv<'a>(
		&self,
		buf: &'a mut impl io::Buffer,
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.stream.recv(buf.borrow_mut()).await {
			Ok(x) => x,
			Err(RecvError::ConnectionClosed) => return Ok(None),
			Err(RecvError::Other(err)) => return Err(ServerError::RecvError(err)),
		};
		let v_minor = self.version.minor();
		Ok(Some(FuseRequest::new(buf, recv_len, v_minor)?))
	}
}

impl<S: io::AsyncOutputStream> AsyncFuseConnection<S> {
	pub async fn reply_ok(
		&self,
		request_id: u64,
		reply: &impl Reply,
	) -> Result<(), io::SendError<S::Error>> {
		reply.send_async(
			&self.stream,
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
		let send = AsyncSendOnce::new(&self.stream);
		let enc = ReplyEncoder::new(send, request_id);
		enc.encode_error(error).await?;
		Ok(())
	}
}

fn handshake<E>(
	recv_buf: &impl Buffer,
	recv_len: usize,
	init_fn: &mut impl FnMut(&FuseInitRequest) -> FuseInitResponse,
) -> Result<(FuseInitResponse, u64, bool), ServerError<E>> {
	let v_latest = io::ProtocolVersion::LATEST;
	let v_minor = v_latest.minor();

	let request_buf = RequestBuf::new(recv_buf, recv_len)?;
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
