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

#![cfg_attr(not(any(doc, feature = "std")), no_std)]

#[macro_use]
mod internal;

mod error;

pub mod client;

pub mod server;

pub mod os {
	#[cfg(any(doc, target_os = "freebsd"))]
	pub mod freebsd;

	#[cfg(not(any(doc, target_os = "freebsd")))]
	#[deprecated = "doc stub"]
	pub mod freebsd {
		#[deprecated = "doc stub"]
		pub struct MountOptions<'a> { _p: &'a () }
	}

	#[cfg(any(doc, target_os = "linux"))]
	pub mod linux;

	#[cfg(not(any(doc, target_os = "linux")))]
	#[deprecated = "doc stub"]
	pub mod linux {
		#[deprecated = "doc stub"]
		pub struct MountOptions<'a> { _p: &'a () }
	}
}

pub mod io;

pub use io::ServerSocket;

pub use crate::error::Error;

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

pub const MIN_READ_BUFFER: usize = internal::fuse_kernel::FUSE_MIN_READ_BUFFER;

/// A version of the FUSE protocol.
///
/// FUSE protocol versions are a (major, minor) version tuple, but FUSE does
/// not use these terms in their common meaning. Backwards compatibility is
/// freely broken in "minor" releases, and the major version is only used
/// during initial connection setup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
	major: u32,
	minor: u32,
}

impl Version {
	const LATEST: Version = Version {
		major: internal::fuse_kernel::FUSE_KERNEL_VERSION,
		minor: internal::fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
	};

	#[inline]
	pub const fn new(major: u32, minor: u32) -> Version {
		Version { major, minor }
	}

	#[inline]
	pub const fn major(&self) -> u32 {
		self.major
	}

	#[inline]
	pub const fn minor(&self) -> u32 {
		self.minor
	}
}
