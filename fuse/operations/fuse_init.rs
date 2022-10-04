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

//! Implements the `FUSE_INIT` operation.

use core::fmt;
use core::marker::PhantomData;

use crate::Version;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// FuseInitRequest {{{

/// Request type for `FUSE_INIT`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_INIT` operation.
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

request_try_from! { FuseInitRequest : fuse }

impl decode::Sealed for FuseInitRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for FuseInitRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_INIT)?;

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
		if raw_v7p1.minor < 6
			|| raw_v7p1.major != fuse_kernel::FUSE_KERNEL_VERSION
		{
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

		let raw: &'a fuse_kernel::fuse_init_in = dec.next_sized()?;
		let mut flags = u64::from(raw.flags);
		flags |= u64::from(raw.flags2) << 32;
		Ok(FuseInitRequest {
			phantom: PhantomData,
			version: Version::new(raw.major, raw.minor),
			max_readahead: raw.max_readahead,
			flags: FuseInitFlags { bits: flags },
		})
	}
}

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
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_INIT` operation.
pub struct FuseInitResponse {
	raw: fuse_kernel::fuse_init_out,
	flags: FuseInitFlags,
}

impl FuseInitResponse {
	#[must_use]
	pub fn new() -> FuseInitResponse {
		Self {
			raw: fuse_kernel::fuse_init_out {
				major: 0,
				minor: 0,
				max_readahead: 0,
				flags: 0,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
				time_gran: 0,
				max_pages: 0,
				map_alignment: 0,
				flags2: 0,
				unused: [0; 7],
			},
			flags: FuseInitFlags::new(),
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
		self.flags
	}

	#[must_use]
	pub fn mut_flags(&mut self) -> &mut FuseInitFlags {
		&mut self.flags
	}

	pub fn set_flags(&mut self, flags: FuseInitFlags) {
		self.flags = flags;
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

response_send_funcs!(FuseInitResponse);

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

#[repr(C)]
struct fuse_init_out_v7p1 {
	major: u32,
	minor: u32,
}

#[repr(C)]
struct fuse_init_out_v7p5 {
	major: u32,
	minor: u32,
	max_readahead: u32,        // v7.6
	flags: u32,                // v7.6
	max_background: u16,       // v7.6
	congestion_threshold: u16, // v7.6
	max_write: u32,
}

impl FuseInitResponse {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		if self.raw.minor >= 23 {
			let mut out = self.raw;
			out.flags = (self.flags.bits & u64::from(u32::MAX)) as u32;
			out.flags2 = (self.flags.bits >> 32) as u32;
			return enc.encode_sized(&out);
		}

		if self.raw.minor >= 5 {
			return enc.encode_sized(&fuse_init_out_v7p5 {
				major: self.raw.major,
				minor: self.raw.minor,
				max_readahead: self.raw.max_readahead,
				flags: (self.flags.bits & u64::from(u32::MAX)) as u32,
				max_background: self.raw.max_background,
				congestion_threshold: self.raw.congestion_threshold,
				max_write: self.raw.max_write,
			});
		}

		enc.encode_sized(&fuse_init_out_v7p1 {
			major: self.raw.major,
			minor: self.raw.minor,
		})
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
	use crate::internal::fuse_kernel;

	bitflags!(FuseInitFlag, FuseInitFlags, u64, {
		ASYNC_READ = fuse_kernel::FUSE_ASYNC_READ;
		POSIX_LOCKS = fuse_kernel::FUSE_POSIX_LOCKS;
		FILE_OPS = fuse_kernel::FUSE_FILE_OPS;
		ATOMIC_O_TRUNC = fuse_kernel::FUSE_ATOMIC_O_TRUNC;
		EXPORT_SUPPORT = fuse_kernel::FUSE_EXPORT_SUPPORT;
		BIG_WRITES = fuse_kernel::FUSE_BIG_WRITES;
		DONT_MASK = fuse_kernel::FUSE_DONT_MASK;
		SPLICE_WRITE = fuse_kernel::FUSE_SPLICE_WRITE;
		SPLICE_MOVE = fuse_kernel::FUSE_SPLICE_MOVE;
		SPLICE_READ = fuse_kernel::FUSE_SPLICE_READ;
		FLOCK_LOCKS = fuse_kernel::FUSE_FLOCK_LOCKS;
		HAS_IOCTL_DIR = fuse_kernel::FUSE_HAS_IOCTL_DIR;
		AUTO_INVAL_DATA = fuse_kernel::FUSE_AUTO_INVAL_DATA;
		DO_READDIRPLUS = fuse_kernel::FUSE_DO_READDIRPLUS;
		READDIRPLUS_AUTO = fuse_kernel::FUSE_READDIRPLUS_AUTO;
		ASYNC_DIO = fuse_kernel::FUSE_ASYNC_DIO;
		WRITEBACK_CACHE = fuse_kernel::FUSE_WRITEBACK_CACHE;
		NO_OPEN_SUPPORT = fuse_kernel::FUSE_NO_OPEN_SUPPORT;
		PARALLEL_DIROPS = fuse_kernel::FUSE_PARALLEL_DIROPS;
		HANDLE_KILLPRIV = fuse_kernel::FUSE_HANDLE_KILLPRIV;
		POSIX_ACL = fuse_kernel::FUSE_POSIX_ACL;
		ABORT_ERROR = fuse_kernel::FUSE_ABORT_ERROR;
		MAX_PAGES = fuse_kernel::FUSE_MAX_PAGES;
		CACHE_SYMLINKS = fuse_kernel::FUSE_CACHE_SYMLINKS;
		NO_OPENDIR_SUPPORT = fuse_kernel::FUSE_NO_OPENDIR_SUPPORT;
		EXPLICIT_INVAL_DATA = fuse_kernel::FUSE_EXPLICIT_INVAL_DATA;
		MAP_ALIGNMENT = fuse_kernel::FUSE_MAP_ALIGNMENT;
		SUBMOUNTS = fuse_kernel::FUSE_SUBMOUNTS;
		HANDLE_KILLPRIV_V2 = fuse_kernel::FUSE_HANDLE_KILLPRIV_V2;
		SETXATTR_EXT = fuse_kernel::FUSE_SETXATTR_EXT;
		INIT_EXT = fuse_kernel::FUSE_INIT_EXT;
		INIT_RESERVED = fuse_kernel::FUSE_INIT_RESERVED;
		SECURITY_CTX = fuse_kernel::FUSE_SECURITY_CTX;
		HAS_INODE_DAX = fuse_kernel::FUSE_HAS_INODE_DAX;
	});
}

// }}}
