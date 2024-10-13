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

use core::fmt;
use core::marker::PhantomData;

use crate::Version;
use crate::kernel;

// FuseInitRequest {{{

/// Request type for `FUSE_INIT`.
#[derive(Clone, Copy)]
pub struct FuseInitRequest<'a> {
	phantom: PhantomData<&'a ()>,
	version: Version,
	max_readahead: u32,
	flags: FuseInitFlags,
}

#[repr(C)]
struct fuse_init_in_v7p1 {
	major: u32,
	minor: u32,
}

#[repr(C)]
struct fuse_init_in_v7p6 {
	pub major:         u32,
	pub minor:         u32,
	pub max_readahead: u32,
	pub flags:         u32,
}

impl FuseInitRequest<'_> {
	#[must_use]
	pub fn version(&self) -> Version {
		self.version
	}

	#[must_use]
	pub fn max_readahead(&self) -> u32 {
		self.max_readahead
	}

	#[must_use]
	pub fn flags(&self) -> FuseInitFlags {
		self.flags
	}
}

try_from_fuse_request!(FuseInitRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_INIT)?;

	// There are two cases where we can't read past the version fields:
	//
	// * Very old protocol versions have a smaller init frame, containing
	//   only the (major, minor) version tuple. Trying to read the modern
	//   frame size would cause EOF.
	//
	// * Mismatch in the major version can't be handled at the request
	//   parsing layer. Per the version negotiation docs, a newer major
	//   version from the kernel should be rejected by sending a response
	//   containing the library's major version.
	let raw_v7p1: &'a fuse_init_in_v7p1 = dec.peek_sized()?;
	if raw_v7p1.minor < 6 || raw_v7p1.major != kernel::FUSE_KERNEL_VERSION {
		return Ok(FuseInitRequest {
			phantom: PhantomData,
			version: Version::new(raw_v7p1.major, raw_v7p1.minor),
			max_readahead: 0,
			flags: FuseInitFlags::new(),
		});
	}

	if raw_v7p1.minor < 36 {
		let raw: &'a fuse_init_in_v7p6 = dec.next_sized()?;
		return Ok(FuseInitRequest {
			phantom: PhantomData,
			version: Version::new(raw.major, raw.minor),
			max_readahead: raw.max_readahead,
			flags: FuseInitFlags {
				bits: u64::from(raw.flags),
			},
		});
	}

	let raw: &'a kernel::fuse_init_in = dec.next_sized()?;
	let mut flags = u64::from(raw.flags);
	flags |= u64::from(raw.flags2) << 32;
	Ok(FuseInitRequest {
		phantom: PhantomData,
		version: Version::new(raw.major, raw.minor),
		max_readahead: raw.max_readahead,
		flags: FuseInitFlags { bits: flags },
	})
});

impl fmt::Debug for FuseInitRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FuseInitRequest")
			.field("version", &self.version)
			.field("max_readahead", &self.max_readahead)
			.field("flags", &self.flags)
			.finish()
	}
}

// }}}

// FuseInitResponse {{{

/// Response type for `FUSE_INIT`.
pub struct FuseInitResponse {
	pub(crate) raw: kernel::fuse_init_out,
}

impl FuseInitResponse {
	#[must_use]
	pub fn new() -> FuseInitResponse {
		Self {
			raw: kernel::fuse_init_out::new(),
		}
	}

	#[must_use]
	pub fn version(&self) -> Version {
		Version::new(self.raw.major, self.raw.minor)
	}

	pub fn set_version(&mut self, v: Version) {
		self.raw.major = v.major();
		self.raw.minor = v.minor();
	}

	#[must_use]
	pub fn flags(&self) -> FuseInitFlags {
		let mut bits = u64::from(self.raw.flags);
		bits |= u64::from(self.raw.flags2) << 32;
		FuseInitFlags { bits }
	}

	pub fn set_flags(&mut self, flags: FuseInitFlags) {
		self.raw.flags = (flags.bits & u64::from(u32::MAX)) as u32;
		self.raw.flags2 = (flags.bits >> 32) as u32;
	}

	#[inline]
	pub fn update_flags(&mut self, f: impl FnOnce(&mut FuseInitFlags)) {
		let mut flags = self.flags();
		f(&mut flags);
		self.set_flags(flags)
	}

	#[must_use]
	pub fn max_readahead(&self) -> u32 {
		self.raw.max_readahead
	}

	pub fn set_max_readahead(&mut self, max_readahead: u32) {
		self.raw.max_readahead = max_readahead;
	}

	#[must_use]
	pub fn max_background(&self) -> u16 {
		self.raw.max_background
	}

	pub fn set_max_background(&mut self, max_background: u16) {
		self.raw.max_background = max_background;
	}

	#[must_use]
	pub fn congestion_threshold(&self) -> u16 {
		self.raw.congestion_threshold
	}

	pub fn set_congestion_threshold(&mut self, congestion_threshold: u16) {
		self.raw.congestion_threshold = congestion_threshold;
	}

	#[must_use]
	pub fn max_write(&self) -> u32 {
		self.raw.max_write
	}

	pub fn set_max_write(&mut self, max_write: u32) {
		self.raw.max_write = max_write;
	}

	#[must_use]
	pub fn time_granularity(&self) -> u32 {
		self.raw.time_gran
	}

	pub fn set_time_granularity(&mut self, granularity: u32) {
		self.raw.time_gran = granularity;
	}
}

impl fmt::Debug for FuseInitResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FuseInitResponse")
			.field("max_readahead", &self.max_readahead())
			.field("flags", &self.flags())
			.field("max_background", &self.max_background())
			.field("congestion_threshold", &self.congestion_threshold())
			.field("max_write", &self.max_write())
			.field("time_granularity", &self.time_granularity())
			.finish()
	}
}

// }}}

// FuseInitFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FuseInitFlags {
	bits: u64,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FuseInitFlag {
	mask: u64,
}

mod flags {
	use crate::kernel;

	bitflags!(FuseInitFlag, FuseInitFlags, u64, {
		ASYNC_READ = kernel::FUSE_ASYNC_READ;
		POSIX_LOCKS = kernel::FUSE_POSIX_LOCKS;
		FILE_OPS = kernel::FUSE_FILE_OPS;
		ATOMIC_O_TRUNC = kernel::FUSE_ATOMIC_O_TRUNC;
		EXPORT_SUPPORT = kernel::FUSE_EXPORT_SUPPORT;
		BIG_WRITES = kernel::FUSE_BIG_WRITES;
		DONT_MASK = kernel::FUSE_DONT_MASK;
		SPLICE_WRITE = kernel::FUSE_SPLICE_WRITE;
		SPLICE_MOVE = kernel::FUSE_SPLICE_MOVE;
		SPLICE_READ = kernel::FUSE_SPLICE_READ;
		FLOCK_LOCKS = kernel::FUSE_FLOCK_LOCKS;
		HAS_IOCTL_DIR = kernel::FUSE_HAS_IOCTL_DIR;
		AUTO_INVAL_DATA = kernel::FUSE_AUTO_INVAL_DATA;
		DO_READDIRPLUS = kernel::FUSE_DO_READDIRPLUS;
		READDIRPLUS_AUTO = kernel::FUSE_READDIRPLUS_AUTO;
		ASYNC_DIO = kernel::FUSE_ASYNC_DIO;
		WRITEBACK_CACHE = kernel::FUSE_WRITEBACK_CACHE;
		NO_OPEN_SUPPORT = kernel::FUSE_NO_OPEN_SUPPORT;
		PARALLEL_DIROPS = kernel::FUSE_PARALLEL_DIROPS;
		HANDLE_KILLPRIV = kernel::FUSE_HANDLE_KILLPRIV;
		POSIX_ACL = kernel::FUSE_POSIX_ACL;
		ABORT_ERROR = kernel::FUSE_ABORT_ERROR;
		MAX_PAGES = kernel::FUSE_MAX_PAGES;
		CACHE_SYMLINKS = kernel::FUSE_CACHE_SYMLINKS;
		NO_OPENDIR_SUPPORT = kernel::FUSE_NO_OPENDIR_SUPPORT;
		EXPLICIT_INVAL_DATA = kernel::FUSE_EXPLICIT_INVAL_DATA;
		MAP_ALIGNMENT = kernel::FUSE_MAP_ALIGNMENT;
		SUBMOUNTS = kernel::FUSE_SUBMOUNTS;
		HANDLE_KILLPRIV_V2 = kernel::FUSE_HANDLE_KILLPRIV_V2;
		SETXATTR_EXT = kernel::FUSE_SETXATTR_EXT;
		INIT_EXT = kernel::FUSE_INIT_EXT;
		INIT_RESERVED = kernel::FUSE_INIT_RESERVED;
		SECURITY_CTX = kernel::FUSE_SECURITY_CTX;
		HAS_INODE_DAX = kernel::FUSE_HAS_INODE_DAX;
	});
}

// }}}
