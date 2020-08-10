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

#![cfg_attr(doc, feature(doc_cfg))]
#![feature(asm)]

#[macro_use]
mod internal;

mod fuse_handlers;
pub use self::fuse_handlers::*;

mod fuse_server;
pub use self::fuse_server::*;

mod server;
pub use self::server::*;

pub use crate::internal::types::ProtocolVersion;

pub mod os {
	#[cfg(any(target_os = "linux", doc))]
	#[cfg_attr(doc, doc(cfg(target_os = "linux")))]
	pub mod linux {
		mod linux_mount_options;
		pub use self::linux_mount_options::*;
	}
}

pub mod protocol;
pub use self::protocol::*;
