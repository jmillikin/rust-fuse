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

use std::path;

use super::fuse_mount::FuseMount;
use crate::channel::Channel;
use crate::fuse_handlers::FuseHandlers;
use crate::fuse_server::{self, FuseServer};

#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub struct FuseServerBuilder<Mount, Handlers> {
	mount_target: path::PathBuf,
	mount: Mount,
	handlers: Handlers,
}

impl<H> FuseServerBuilder<(), H> {
	pub fn new(
		mount_target: impl AsRef<path::Path>,
		handlers: H,
	) -> FuseServerBuilder<(), H> {
		FuseServerBuilder {
			mount_target: path::PathBuf::from(mount_target.as_ref()),
			mount: (),
			handlers,
		}
	}
}

impl<_M, H> FuseServerBuilder<_M, H> {
	pub fn set_mount<M>(self, mount: M) -> FuseServerBuilder<M, H> {
		FuseServerBuilder {
			mount_target: self.mount_target,
			mount,
			handlers: self.handlers,
		}
	}
}

impl<M, H> FuseServerBuilder<M, H>
where
	M: FuseMount,
	H: FuseHandlers,
{
	pub fn build(
		self,
	) -> Result<
		FuseServer<M::Channel, H>,
		<<M as FuseMount>::Channel as Channel>::Error,
	> {
		let channel = self.mount.fuse_mount(&self.mount_target)?;
		fuse_server::FuseServerBuilder::new(channel, self.handlers).build()
	}
}
