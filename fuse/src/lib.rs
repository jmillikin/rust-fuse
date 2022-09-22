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

mod io;

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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Opcode(pub u32);

macro_rules! export_opcodes {
	( $( $(#[$meta:meta])* $name:ident , )+ ) => {
		mod fuse_opcode {
			$(
				pub const $name: u32 = crate::internal::fuse_kernel::$name.0;
			)+
		}
		impl Opcode {
			$(
				$(#[$meta])*
				pub const $name: Opcode = Opcode(fuse_opcode::$name);
			)+
		}
	};
}

export_opcodes! {
	FUSE_LOOKUP,
	FUSE_FORGET,
	FUSE_GETATTR,
	FUSE_SETATTR,
	FUSE_READLINK,
	FUSE_SYMLINK,
	FUSE_MKNOD,
	FUSE_MKDIR,
	FUSE_UNLINK,
	FUSE_RMDIR,
	FUSE_RENAME,
	FUSE_LINK,
	FUSE_OPEN,
	FUSE_READ,
	FUSE_WRITE,
	FUSE_STATFS,
	FUSE_RELEASE,
	FUSE_FSYNC,
	FUSE_SETXATTR,
	FUSE_GETXATTR,
	FUSE_LISTXATTR,
	FUSE_REMOVEXATTR,
	FUSE_FLUSH,
	FUSE_INIT,
	FUSE_OPENDIR,
	FUSE_READDIR,
	FUSE_RELEASEDIR,
	FUSE_FSYNCDIR,
	FUSE_GETLK,
	FUSE_SETLK,
	FUSE_SETLKW,
	FUSE_ACCESS,
	FUSE_CREATE,
	FUSE_INTERRUPT,
	FUSE_BMAP,
	FUSE_DESTROY,
	FUSE_IOCTL,
	FUSE_POLL,
	FUSE_NOTIFY_REPLY,
	FUSE_BATCH_FORGET,
	FUSE_FALLOCATE,
	FUSE_READDIRPLUS,
	FUSE_RENAME2,
	FUSE_LSEEK,
	FUSE_COPY_FILE_RANGE,
	FUSE_SETUPMAPPING,
	FUSE_REMOVEMAPPING,
	FUSE_SYNCFS,

	CUSE_INIT,

	CUSE_INIT_BSWAP_RESERVED,
	FUSE_INIT_BSWAP_RESERVED,
}
