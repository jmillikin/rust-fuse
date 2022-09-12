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

#[cfg(feature = "libc_fuse_mount")]
extern crate libc;

#[macro_use]
mod internal;

pub mod client;

pub mod server;

pub mod os {
	#[cfg(any(doc, target_os = "freebsd"))]
	pub mod freebsd;

	#[cfg(any(doc, target_os = "linux"))]
	pub mod linux;

	#[cfg(any(doc, unix))]
	pub mod unix;
}

pub mod io;

pub(crate) struct ErrorCode;

#[allow(dead_code)]
#[cfg(target_os = "linux")]
impl ErrorCode {
	pub(crate) const E2BIG:  linux_errno::Error = linux_errno::E2BIG;
	pub(crate) const EINTR:  linux_errno::Error = linux_errno::EINTR;
	pub(crate) const EIO:    linux_errno::Error = linux_errno::EIO;
	pub(crate) const ENOENT: linux_errno::Error = linux_errno::ENOENT;
	pub(crate) const ENOSYS: linux_errno::Error = linux_errno::ENOSYS;
	pub(crate) const ERANGE: linux_errno::Error = linux_errno::ERANGE;

	pub(crate) const EINTR_I32:  i32 = linux_errno::EINTR.get()  as i32;
	pub(crate) const ENODEV_I32: i32 = linux_errno::ENODEV.get() as i32;
	pub(crate) const ENOENT_I32: i32 = linux_errno::ENOENT.get() as i32;
}

#[allow(dead_code)]
#[cfg(target_os = "freebsd")]
impl ErrorCode {
	pub(crate) const E2BIG:  freebsd_errno::Error = freebsd_errno::E2BIG;
	pub(crate) const EINTR:  freebsd_errno::Error = freebsd_errno::EINTR;
	pub(crate) const EIO:    freebsd_errno::Error = freebsd_errno::EIO;
	pub(crate) const ENOENT: freebsd_errno::Error = freebsd_errno::ENOENT;
	pub(crate) const ENOSYS: freebsd_errno::Error = freebsd_errno::ENOSYS;
	pub(crate) const ERANGE: freebsd_errno::Error = freebsd_errno::ERANGE;

	pub(crate) const EINTR_I32:  i32 = freebsd_errno::EINTR.get();
	pub(crate) const ENODEV_I32: i32 = freebsd_errno::ENODEV.get();
	pub(crate) const ENOENT_I32: i32 = freebsd_errno::ENOENT.get();
}

pub mod protocol;
pub use crate::protocol::*;

pub use self::protocol::common::{
	FileMode,
	FileType,
	Lock,
	LockRange,
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
