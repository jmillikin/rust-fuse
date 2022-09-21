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
use crate::protocol::fuse_init::{
	FuseInitRequest,
	FuseInitResponse,
};
use crate::server::ServerError;

pub struct FuseConnectionBuilder<S> {
	socket: S,
}

impl<S> FuseConnectionBuilder<S> {
	pub fn new(socket: S) -> Self {
		Self { socket }
	}
}

impl<S: io::FuseServerSocket> FuseConnectionBuilder<S> {
	pub fn build(self) -> Result<FuseConnection<S>, ServerError<S::Error>> {
		FuseConnection::new(self.socket, |_request| {
			FuseInitResponse::new()
		})
	}
}

pub struct FuseConnection<S> {
	pub(crate) socket: S,
	pub(crate) init_response: FuseInitResponse,
}

impl<S: io::FuseServerSocket> FuseConnection<S> {
	pub fn new(
		mut socket: S,
		init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
	) -> Result<Self, ServerError<S::Error>> {
		let init_response = super::fuse_init(&mut socket, init_fn)?;
		Ok(Self {
			socket,
			init_response,
		})
	}
}
