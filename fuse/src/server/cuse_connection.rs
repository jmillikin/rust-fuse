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
use crate::io::encode::{AsyncSendOnce, SyncSendOnce};
use crate::protocol::cuse_init::{
	CuseDeviceName,
	CuseInitFlags,
	CuseInitRequest,
	CuseInitResponse,
};
use crate::server::{CuseRequest, CuseRequestBuilder, ServerError};
use crate::server::connection::negotiate_version;

pub struct CuseConnectionBuilder<'a, S> {
	socket: S,
	device_name: &'a CuseDeviceName,
	dev_major: u32,
	dev_minor: u32,
	max_read: u32,
	max_write: u32,
	flags: CuseInitFlags,
}

impl<'a, S> CuseConnectionBuilder<'a, S> {
	pub fn new(socket: S, device_name: &'a CuseDeviceName) -> Self {
		Self {
			socket,
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

impl<S: io::CuseServerSocket> CuseConnectionBuilder<'_, S> {
	pub fn build(self) -> Result<CuseConnection<S>, ServerError<S::Error>> {
		let dev_major = self.dev_major;
		let dev_minor = self.dev_minor;
		let max_read = self.max_read;
		let max_write = self.max_write;
		let flags = self.flags;
		CuseConnection::new(self.socket, self.device_name, |_request| {
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

impl<S: io::AsyncCuseServerSocket> CuseConnectionBuilder<'_, S> {
	pub async fn build_async(
		self,
	) -> Result<AsyncCuseConnection<S>, ServerError<S::Error>> {
		let dev_major = self.dev_major;
		let dev_minor = self.dev_minor;
		let max_read = self.max_read;
		let max_write = self.max_write;
		let flags = self.flags;
		AsyncCuseConnection::new(self.socket, self.device_name, |_request| {
			let mut reply = CuseInitResponse::new();
			reply.set_dev_major(dev_major);
			reply.set_dev_minor(dev_minor);
			reply.set_max_read(max_read);
			reply.set_max_write(max_write);
			*reply.flags_mut() = flags;
			reply
		}).await
	}
}

pub struct CuseConnection<S> {
	socket: S,
	version: Version,
}

impl<S: io::CuseServerSocket> CuseConnection<S> {
	pub fn new(
		socket: S,
		device_name: &CuseDeviceName,
		init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_reply = Self::handshake(&socket, device_name, init_fn)?;
		Ok(Self {
			socket,
			version: init_reply.version(),
		})
	}

	pub(crate) fn socket(&self) -> &S {
		&self.socket
	}

	fn handshake(
		socket: &S,
		device_name: &CuseDeviceName,
		mut init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse,
	) -> Result<CuseInitResponse, ServerError<S::Error>> {
		let mut buf = io::ArrayBuffer::new();
		let buf = buf.borrow_mut();

		loop {
			let recv = socket.recv(buf)?;
			let (reply, request_id, ok) = handshake(&buf, recv, &mut init_fn)?;

			reply.encode(
				SyncSendOnce::new(socket),
				request_id,
				if ok { Some(device_name.as_bytes()) } else { None },
			)?;

			if ok {
				return Ok(reply);
			}
		}
	}
}

impl<S> CuseConnection<S> {
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

impl<S: io::CuseServerSocket> CuseConnection<S> {
	pub fn recv<'a>(
		&self,
		buf: &'a mut [u8],
	) -> Result<Option<CuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf) {
			Ok(x) => x,
			Err(err) => return Err(ServerError::from(err)),
		};
		let request = CuseRequestBuilder::new()
			.version(self.version)
			.build(&buf[..recv_len])?;
		Ok(Some(request))
	}
}

pub struct AsyncCuseConnection<S> {
	socket: S,
	version: Version,
}

impl<S: io::AsyncCuseServerSocket> AsyncCuseConnection<S> {
	pub async fn new(
		socket: S,
		device_name: &CuseDeviceName,
		init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_reply = Self::handshake(&socket, device_name, init_fn).await?;
		Ok(Self {
			socket,
			version: init_reply.version(),
		})
	}

	async fn handshake(
		socket: &S,
		device_name: &CuseDeviceName,
		mut init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse,
	) -> Result<CuseInitResponse, ServerError<S::Error>> {
		let mut buf = io::ArrayBuffer::new();
		let buf = buf.borrow_mut();

		loop {
			let recv = socket.recv(buf).await?;
			let (reply, req_id, done) = handshake(&buf, recv, &mut init_fn)?;

			reply.encode(
				AsyncSendOnce::new(socket),
				req_id,
				if done { Some(device_name.as_bytes()) } else { None },
			).await?;

			if done {
				return Ok(reply);
			}
		}
	}
}

impl<S> AsyncCuseConnection<S> {
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

impl<S: io::AsyncCuseServerSocket> AsyncCuseConnection<S> {
	pub async fn recv<'a>(
		&self,
		buf: &'a mut [u8],
	) -> Result<Option<CuseRequest<'a>>, ServerError<S::Error>> {
		let recv_len = match self.socket.recv(buf).await {
			Ok(x) => x,
			Err(RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(RecvError::Other(err)) => return Err(ServerError::RecvError(err)),
		};
		let request = CuseRequestBuilder::new()
			.version(self.version)
			.build(&buf[..recv_len])?;
		Ok(Some(request))
	}
}

fn handshake<E>(
	recv_buf: &[u8],
	recv_len: usize,
	init_fn: &mut impl FnMut(&CuseInitRequest) -> CuseInitResponse,
) -> Result<(CuseInitResponse, u64, bool), ServerError<E>> {
	let v_latest = Version::LATEST;
	let v_minor = v_latest.minor();

	let request_buf = RequestBuf::new(&recv_buf[..recv_len])?;
	let init_request = CuseInitRequest::from_cuse_request(&CuseRequest {
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
			let mut init_reply = CuseInitResponse::new();
			init_reply.set_version(v_latest);
			init_reply
		},
	};

	let request_id = request_buf.header().unique;
	Ok((init_reply, request_id, done))
}
