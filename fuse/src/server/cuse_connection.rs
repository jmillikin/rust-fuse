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
use crate::io::encode::{ReplyEncoder, SyncSendOnce};
use crate::protocol::cuse_init::{
	CuseDeviceName,
	CuseInitFlags,
	CuseInitRequest,
	CuseInitResponse,
};
use crate::server::{CuseRequest, Reply, ReplyInfo};
use crate::server::connection::negotiate_version;

pub struct CuseConnectionBuilder<'a, Stream> {
	stream: Stream,
	device_name: &'a CuseDeviceName,
	dev_major: u32,
	dev_minor: u32,
	max_read: u32,
	max_write: u32,
	flags: CuseInitFlags,
}

impl<'a, Stream> CuseConnectionBuilder<'a, Stream> {
	pub fn new(stream: Stream, device_name: &'a CuseDeviceName) -> Self {
		Self {
			stream,
			device_name,
			dev_major: 0,
			dev_minor: 0,
			max_read: 0,
			max_write: 0,
			flags: CuseInitFlags::new(),
		}
	}
}

impl<S> CuseConnectionBuilder<'_, S> {
	pub fn device_number(mut self, major: u32, minor: u32) -> Self {
		self.dev_major = major;
		self.dev_minor = minor;
		self
	}

	pub fn max_read(mut self, max_read: u32) -> Self {
		self.max_read = max_read;
		self
	}

	pub fn max_write(mut self, max_write: u32) -> Self {
		self.max_write = max_write;
		self
	}

	pub fn unrestricted_ioctl(mut self, x: bool) -> Self {
		self.flags.unrestricted_ioctl = x;
		self
	}
}

impl<S, E> CuseConnectionBuilder<'_, S>
where
	S: io::InputStream<Error = E> + io::OutputStream<Error = E>,
{
	pub fn build(self) -> Result<CuseConnection<S>, io::Error<E>> {
		let dev_major = self.dev_major;
		let dev_minor = self.dev_minor;
		let max_read = self.max_read;
		let max_write = self.max_write;
		let flags = self.flags;
		CuseConnection::new(self.stream, self.device_name, |_request| {
			let mut reply = CuseInitResponse::new();
			reply.set_dev_major(dev_major);
			reply.set_dev_minor(dev_minor);
			reply.set_max_read(max_read);
			reply.set_max_write(max_write);
			*reply.flags_mut() = flags;
			reply
		})
	}
}

pub struct CuseConnection<Stream> {
	stream: Stream,
	version: io::ProtocolVersion,
}

impl<S, E> CuseConnection<S>
where
	S: io::InputStream<Error = E> + io::OutputStream<Error = E>,
{
	pub fn new(
		stream: S,
		device_name: &CuseDeviceName,
		init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse,
	) -> Result<Self, io::Error<E>> {
		let init_reply = Self::handshake(&stream, device_name, init_fn)?;
		Ok(Self {
			stream,
			version: init_reply.version(),
		})
	}

	fn handshake(
		stream: &S,
		device_name: &CuseDeviceName,
		mut init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse,
	) -> Result<CuseInitResponse, io::Error<E>> {
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
			let init_request = CuseInitRequest::decode(request_buf, v_minor)?;
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
					let mut init_reply = CuseInitResponse::new();
					init_reply.set_version(v_latest);
					init_reply
				},
			};

			init_reply.encode(
				SyncSendOnce::new(stream),
				request_id,
				if done { Some(device_name.as_bytes()) } else { None },
			).map_err(|err| io::Error::SendFail(err))?;

			if done {
				return Ok(init_reply);
			}
		}
	}
}

impl<S> CuseConnection<S> {
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

impl<S: io::InputStream> CuseConnection<S> {
	pub fn recv<'a>(
		&self,
		buf: &'a mut impl io::Buffer,
	) -> Result<Option<CuseRequest<'a>>, io::Error<S::Error>> {
		let recv_len = match self.stream.recv(buf.borrow_mut()) {
			Ok(Some(x)) => x,
			Ok(None) => return Ok(None),
			Err(err) => return Err(io::Error::RecvFail(err)),
		};
		let v_minor = self.version.minor();
		Ok(Some(CuseRequest::new(buf, recv_len, v_minor)?))
	}
}

impl<S: io::OutputStream> CuseConnection<S> {
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
