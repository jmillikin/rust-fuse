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

use crate::io;
use crate::protocol::cuse_init::{
	CuseDeviceName,
	CuseInitFlags,
	CuseInitRequest,
	CuseInitResponse,
};
use crate::server::ServerError;

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
		let device_name = self.device_name;
		let dev_major = self.dev_major;
		let dev_minor = self.dev_minor;
		let max_read = self.max_read;
		let max_write = self.max_write;
		let flags = self.flags;
		CuseConnection::new(self.socket, |_request| {
			let mut reply = CuseInitResponse::new(device_name);
			reply.set_dev_major(dev_major);
			reply.set_dev_minor(dev_minor);
			reply.set_max_read(max_read);
			reply.set_max_write(max_write);
			*reply.flags_mut() = flags;
			reply
		})
	}
}

pub struct CuseConnection<S> {
	pub(crate) socket: S,
	pub(crate) init_response: CuseInitResponse<'static>,
}

impl<S: io::CuseServerSocket> CuseConnection<S> {
	pub fn new<'a>(
		mut socket: S,
		init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse<'a>,
	) -> Result<Self, ServerError<S::Error>> {
		let init_response = super::cuse_init(&mut socket, init_fn)?;
		Ok(Self {
			socket,
			init_response: init_response.drop_name(),
		})
	}
}
