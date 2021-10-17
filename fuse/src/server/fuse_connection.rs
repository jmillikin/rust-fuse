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

use core::num::NonZeroU16;

use crate::io::{self, Buffer};
use crate::io::decode::{DecodeRequest, RequestBuf};
use crate::io::encode::{EncodeReply, ReplyEncoder, SyncSendOnce};
use crate::protocol::fuse_init::{
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server::{FuseRequest, Reply, ReplyInfo};
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
	pub fn build(self) -> Result<FuseConnection<S>, io::Error<E>> {
		let flags = self.flags;
		FuseConnection::new(self.stream, |_request| {
			let mut reply = FuseInitResponse::new();
			*reply.flags_mut() = flags;
			reply
		})
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
	) -> Result<Self, io::Error<E>> {
		let init_reply = Self::handshake(&stream, init_fn)?;
		Ok(Self {
			stream,
			version: init_reply.version(),
		})
	}

	fn handshake(
		stream: &S,
		mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<FuseInitResponse, io::Error<E>> {
		let mut read_buf = io::ArrayBuffer::new();
		let v_latest = io::ProtocolVersion::LATEST;
		let v_minor = v_latest.minor();

		loop {
			let recv_len = match stream.recv(read_buf.borrow_mut()) {
				Ok(Some(x)) => x,
				Ok(None) => {
					// TODO
					return Err(io::RequestError::UnexpectedEof.into());
				},
				Err(err) => return Err(io::Error::RecvFail(err)),
			};
			let request_buf = RequestBuf::new(&read_buf, recv_len)?;
			let init_request = FuseInitRequest::decode(request_buf, v_minor)?;
			let request_id = request_buf.header().unique;

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

			init_reply.encode(
				SyncSendOnce::new(stream),
				request_id,
				// FuseInitResponse always encodes with its own version
				v_minor,
			).map_err(|err| io::Error::SendFail(err))?;

			if done {
				return Ok(init_reply);
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
	) -> Result<Option<FuseRequest<'a>>, io::Error<S::Error>> {
		let recv_len = match self.stream.recv(buf.borrow_mut()) {
			Ok(Some(x)) => x,
			Ok(None) => return Ok(None),
			Err(err) => return Err(io::Error::RecvFail(err)),
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
	) -> Result<(), io::Error<S::Error>> {
		let res = reply.send(
			&self.stream,
			ReplyInfo {
				request_id,
				version_minor: self.version.minor(),
			},
		);
		res.map_err(|err| io::Error::SendFail(err))
	}

	pub fn reply_err(
		&self,
		request_id: u64,
		error_code: NonZeroU16,
	) -> Result<(), io::Error<S::Error>> {
		let send = SyncSendOnce::new(&self.stream);
		let enc = ReplyEncoder::new(send, request_id);
		let res = enc.encode_error(error_code);
		res.map_err(|err| io::Error::SendFail(err))
	}
}
