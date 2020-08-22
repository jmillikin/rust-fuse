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

#![cfg(feature = "std")]

#[cfg(feature = "nightly_syscall_fuse_mount")]
mod linux_syscalls;

mod cuse_server_builder;
pub use self::cuse_server_builder::*;

mod dev_cuse_channel;
pub use self::dev_cuse_channel::*;

mod dev_fuse_channel;
pub use self::dev_fuse_channel::*;

mod fuse_server_builder;
pub use self::fuse_server_builder::*;

#[cfg(any(
	feature = "libc_fuse_mount",
	feature = "nightly_syscall_fuse_mount",
))]
mod fuse_mount;

#[cfg(any(
	feature = "libc_fuse_mount",
	feature = "nightly_syscall_fuse_mount",
))]
pub use self::fuse_mount::*;
