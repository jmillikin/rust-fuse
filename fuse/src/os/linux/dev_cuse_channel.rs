// Copyright 2020 John Millikin and the rust-fuse contributors.
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

use std::{fs, io};

use crate::channel;
use crate::cuse_server;

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub struct DevCuseChannel(channel::FileChannel);

impl DevCuseChannel {
	pub(super) fn new(file: fs::File) -> DevCuseChannel {
		Self(channel::FileChannel::new(file))
	}
}

impl channel::Channel for DevCuseChannel {
	type Error = io::Error;

	fn send(&self, buf: &[u8]) -> Result<(), io::Error> {
		self.0.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), io::Error> {
		self.0.send_vectored(bufs)
	}

	fn receive(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
		self.0.receive(buf)
	}
}

impl cuse_server::CuseServerChannel for DevCuseChannel {
	fn try_clone(&self) -> Result<Self, io::Error> {
		Ok(DevCuseChannel(self.0.try_clone()?))
	}
}
