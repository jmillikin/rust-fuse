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

use std::io;

use crate::channel;
use crate::fuse_server;
use crate::old_server as server;

#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub struct DevFuseChannel(channel::FileChannel);

impl DevFuseChannel {
	#[cfg(any(
		feature = "libc_fuse_mount",
		feature = "nightly_syscall_fuse_mount",
	))]
	pub(super) fn new(file: std::fs::File) -> DevFuseChannel {
		Self(channel::FileChannel::new(file))
	}
}

impl channel::Channel for DevFuseChannel {
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

impl server::ServerChannel for DevFuseChannel {
	fn try_clone(&self) -> Result<Self, io::Error> {
		Ok(DevFuseChannel(self.0.try_clone()?))
	}
}

impl fuse_server::FuseServerChannel for DevFuseChannel {}
