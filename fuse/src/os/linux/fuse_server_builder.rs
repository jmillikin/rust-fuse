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

use std::{io, path};

use super::fuse_mount::{FuseMount, SyscallFuseMount};
use crate::channel::{self, FileChannel};
use crate::fuse_handlers::FuseHandlers;
use crate::fuse_server::{self, FuseServer};

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub struct FuseServerBuilder<Mount, Handlers> {
	mount_target: path::PathBuf,
	mount: Mount,
	handlers: Handlers,
}

impl<Handlers> FuseServerBuilder<SyscallFuseMount, Handlers>
where
	Handlers: FuseHandlers,
{
	pub fn new(
		mount_target: impl AsRef<path::Path>,
		handlers: Handlers,
	) -> FuseServerBuilder<SyscallFuseMount, Handlers> {
		FuseServerBuilder {
			mount_target: path::PathBuf::from(mount_target.as_ref()),
			mount: SyscallFuseMount::new(),
			handlers,
		}
	}
}

impl<Mount, Handlers> FuseServerBuilder<Mount, Handlers>
where
	Mount: FuseMount,
	Handlers: FuseHandlers,
{
	pub fn set_mount(mut self, mount: Mount) -> Self {
		self.mount = mount;
		self
	}

	pub fn build(self) -> io::Result<FuseServer<FuseServerChannel, Handlers>> {
		let file = self.mount.fuse_mount(&self.mount_target)?;
		FuseServer::new(FuseServerChannel(FileChannel::new(file)), self.handlers)
	}
}

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub struct FuseServerChannel(FileChannel);

impl channel::Channel for FuseServerChannel {
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

impl fuse_server::FuseServerChannel for FuseServerChannel {
	fn try_clone(&self) -> Result<Self, io::Error> {
		Ok(FuseServerChannel(self.0.try_clone()?))
	}
}
