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

use crate::channel::Channel;
use crate::fuse_handlers::FuseHandlers;
use crate::fuse_server::{self, FuseServer};
use crate::server;

#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub trait FuseMount {
	type Channel: fuse_server::FuseServerChannel;

	fn fuse_mount(
		self,
		mount_target: &path::Path,
	) -> Result<Self::Channel, <Self::Channel as Channel>::Error>;
}

#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub struct FuseServerBuilder<Mount, Handlers, Hooks> {
	mount_target: path::PathBuf,
	mount: Mount,
	handlers: Handlers,
	hooks: Option<Hooks>,
}

impl<Handlers> FuseServerBuilder<(), Handlers, server::NoopServerHooks> {
	pub fn new(
		mount_target: impl AsRef<path::Path>,
		handlers: Handlers,
	) -> FuseServerBuilder<(), Handlers, server::NoopServerHooks> {
		FuseServerBuilder {
			mount_target: path::PathBuf::from(mount_target.as_ref()),
			mount: (),
			handlers,
			hooks: None,
		}
	}
}

impl<Mount, Handlers, Hooks> FuseServerBuilder<Mount, Handlers, Hooks> {
	pub fn set_hooks<H>(
		self,
		hooks: H,
	) -> FuseServerBuilder<Mount, Handlers, H> {
		FuseServerBuilder {
			mount_target: self.mount_target,
			mount: self.mount,
			handlers: self.handlers,
			hooks: Some(hooks),
		}
	}

	pub fn set_mount<M>(
		self,
		mount: M,
	) -> FuseServerBuilder<M, Handlers, Hooks> {
		FuseServerBuilder {
			mount_target: self.mount_target,
			mount,
			handlers: self.handlers,
			hooks: self.hooks,
		}
	}
}

impl<M, Handlers, Hooks> FuseServerBuilder<M, Handlers, Hooks>
where
	M: FuseMount,
	Handlers: FuseHandlers,
	Hooks: server::ServerHooks,
{
	pub fn build(
		self,
	) -> Result<
		FuseServer<M::Channel, Handlers, Hooks>,
		<<M as FuseMount>::Channel as Channel>::Error,
	> {
		let channel = self.mount.fuse_mount(&self.mount_target)?;
		let mut builder =
			fuse_server::FuseServerBuilder::new(channel, self.handlers);
		if let Some(hooks) = self.hooks {
			builder = builder.set_hooks(hooks);
		}
		builder.build()
	}
}
