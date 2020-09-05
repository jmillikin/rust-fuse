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

#![cfg_attr(not(feature = "std"), no_std)]

#![cfg_attr(doc, feature(doc_cfg))]

// For direct syscalls in `fuse/src/os/linux/syscalls.rs`.
#![cfg_attr(feature = "nightly_syscall_fuse_mount", feature(asm))]

// For `send_vectored` in `fuse/src/channel.rs`.
#![allow(incomplete_features)]
#![cfg_attr(any(
	doc,
	feature = "nightly_impl_channel",
), feature(const_generics))]

#[cfg(feature = "libc_fuse_mount")]
extern crate libc;

#[macro_use]
mod internal;

mod channel;
mod error;

mod cuse_handlers;
pub use self::cuse_handlers::CuseHandlers;

mod cuse_server;
pub use self::cuse_server::{
	CuseDeviceName,
	CuseServer,
	CuseServerBuilder,
	CuseServerExecutor,
};

mod fuse_handlers;
pub use self::fuse_handlers::FuseHandlers;

mod fuse_server;
pub use self::fuse_server::{
	FuseServer,
	FuseServerBuilder,
	FuseServerExecutor,
};

mod server;
pub use self::server::{
	ServerContext,
	ServerHooks,
	Respond,
};

#[cfg(feature = "respond_async")]
pub use self::server::RespondAsync;

pub mod os {
	#[cfg(any(doc, target_os = "linux"))]
	#[cfg_attr(doc, doc(cfg(target_os = "linux")))]
	pub mod linux;
}

pub mod io {
	pub use crate::internal::types::ProtocolVersion;
	
	pub use crate::channel::{Channel, ChannelError};
	pub use crate::server::ServerChannel;

	pub use crate::cuse_server::CuseServerChannel;
	pub use crate::fuse_server::FuseServerChannel;
}

pub use crate::error::{Error, ErrorCode};

pub mod protocol;
pub use crate::protocol::*;

pub use self::protocol::common::{
	FileMode,
	FileType,
	Node,
	NodeAttr,
	NodeId,
	NodeName,
	XattrName,
	NODE_NAME_MAX,
	ROOT_ID,
	XATTR_LIST_MAX,
	XATTR_NAME_MAX,
	XATTR_SIZE_MAX,
};
