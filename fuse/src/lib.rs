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

#![cfg_attr(feature = "no_std", no_std)]

#![cfg_attr(doc, feature(doc_cfg))]

// For direct syscalls in `fuse/src/os/linux/syscalls.rs`.
#![feature(asm)]

// For `send_vectored` in `fuse/src/channel.rs`.
#![allow(incomplete_features)]
#![feature(const_generics)]

#[macro_use]
mod internal;

mod channel;
mod error;

mod cuse_handlers;
pub use self::cuse_handlers::CuseHandlers;

#[cfg(not(feature = "no_std"))]
mod cuse_server;
#[cfg(not(feature = "no_std"))]
pub use self::cuse_server::{CuseDeviceName, CuseServer};

mod fuse_handlers;
pub use self::fuse_handlers::FuseHandlers;

#[cfg(not(feature = "no_std"))]
mod fuse_server;
#[cfg(not(feature = "no_std"))]
pub use self::fuse_server::FuseServer;

mod server;
pub use self::server::*;

pub mod os {
	#[cfg(any(doc, target_os = "linux"))]
	#[cfg_attr(doc, doc(cfg(target_os = "linux")))]
	pub mod linux;
}

pub mod io {
	pub use crate::internal::types::ProtocolVersion;
	
	pub use crate::channel::{Channel, ChannelError};
	pub use crate::error::{Error, ErrorCode};
	
	#[cfg(not(feature = "no_std"))]
	pub use crate::cuse_server::CuseServerChannel;

	#[cfg(not(feature = "no_std"))]
	pub use crate::fuse_server::FuseServerChannel;
}

#[doc(no_inline)]
pub use crate::io::ErrorCode;

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
