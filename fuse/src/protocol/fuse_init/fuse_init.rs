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

use crate::io::ProtocolVersion;
use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "fuse_init_test")]
mod fuse_init_test;

// FuseInitRequest {{{

/// Request type for [`FuseHandlers::fuse_init`].
///
/// [`FuseHandlers::fuse_init`]: ../../trait.FuseHandlers.html#method.fuse_init
pub struct FuseInitRequest<'a> {
	phantom: PhantomData<&'a ()>,
	version: ProtocolVersion,
	max_readahead: u32,
	flags: FuseInitFlags,
}

impl FuseInitRequest<'_> {
	pub fn version(&self) -> ProtocolVersion {
		self.version
	}

	pub fn max_readahead(&self) -> u32 {
		self.max_readahead
	}

	pub fn set_max_readahead(&mut self, max_readahead: u32) {
		self.max_readahead = max_readahead;
	}

	pub fn flags(&self) -> &FuseInitFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut FuseInitFlags {
		&mut self.flags
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

#[repr(C)]
struct fuse_init_in_v7p1 {
	major: u32,
	minor: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for FuseInitRequest<'_> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		debug_assert!(dec.header().opcode == fuse_kernel::FUSE_INIT);

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
				version: ProtocolVersion::new(raw_v7p1.major, raw_v7p1.minor),
				max_readahead: 0,
				flags: FuseInitFlags::from_bits(0),
			});
		}

		let raw: &'a fuse_kernel::fuse_init_in = dec.next_sized()?;
		Ok(FuseInitRequest {
			phantom: PhantomData,
			version: ProtocolVersion::new(raw.major, raw.minor),
			max_readahead: raw.max_readahead,
			flags: FuseInitFlags::from_bits(raw.flags),
		})
	}
}

// }}}

// FuseInitResponse {{{

/// Response type for [`FuseHandlers::fuse_init`].
///
/// [`FuseHandlers::fuse_init`]: ../../trait.FuseHandlers.html#method.fuse_init
pub struct FuseInitResponse {
	raw: fuse_kernel::fuse_init_out,
	flags: FuseInitFlags,
}

impl FuseInitResponse {
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
				unused: [0; 9],
			},
			flags: FuseInitFlags::new(),
		}
	}

	pub(crate) fn version(&self) -> ProtocolVersion {
		ProtocolVersion::new(self.raw.major, self.raw.minor)
	}

	pub(crate) fn set_version(&mut self, v: ProtocolVersion) {
		self.raw.major = v.major();
		self.raw.minor = v.minor();
	}

	pub fn flags(&self) -> &FuseInitFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut FuseInitFlags {
		&mut self.flags
	}

	pub fn max_readahead(&self) -> u32 {
		self.raw.max_readahead
	}

	pub fn set_max_readahead(&mut self, max_readahead: u32) {
		self.raw.max_readahead = max_readahead;
	}

	pub fn max_background(&self) -> u16 {
		self.raw.max_background
	}

	pub fn set_max_background(&mut self, max_background: u16) {
		self.raw.max_background = max_background;
	}

	pub fn congestion_threshold(&self) -> u16 {
		self.raw.congestion_threshold
	}

	pub fn set_congestion_threshold(&mut self, congestion_threshold: u16) {
		self.raw.congestion_threshold = congestion_threshold;
	}

	pub fn max_write(&self) -> u32 {
		self.raw.max_write
	}

	pub fn set_max_write(&mut self, max_write: u32) {
		self.raw.max_write = max_write;
	}

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
			.field("flags", self.flags())
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

impl fuse_io::EncodeResponse for FuseInitResponse {
	fn encode_response<'a, S: io::OutputStream>(
		&'a self,
		enc: fuse_io::ResponseEncoder<S>,
	) -> Result<(), S::Error> {
		if self.raw.minor >= 23 {
			let mut out = self.raw;
			out.flags = self.flags.to_bits();
			return enc.encode_sized(&out);
		}

		if self.raw.minor >= 5 {
			return enc.encode_sized(&fuse_init_out_v7p5 {
				major: self.raw.major,
				minor: self.raw.minor,
				max_readahead: self.raw.max_readahead,
				flags: self.flags.to_bits(),
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

bitflags_struct! {
	pub struct FuseInitFlags(u32);

	fuse_kernel::FUSE_ASYNC_READ: async_read,
	fuse_kernel::FUSE_POSIX_LOCKS: posix_locks,
	fuse_kernel::FUSE_FILE_OPS: file_ops,
	fuse_kernel::FUSE_ATOMIC_O_TRUNC: atomic_o_trunc,
	fuse_kernel::FUSE_EXPORT_SUPPORT: export_support,
	fuse_kernel::FUSE_BIG_WRITES: big_writes,
	fuse_kernel::FUSE_DONT_MASK: dont_mask,
	fuse_kernel::FUSE_SPLICE_WRITE: splice_write,
	fuse_kernel::FUSE_SPLICE_MOVE: splice_move,
	fuse_kernel::FUSE_SPLICE_READ: splice_read,
	fuse_kernel::FUSE_FLOCK_LOCKS: flock_locks,
	fuse_kernel::FUSE_HAS_IOCTL_DIR: has_ioctl_dir,
	fuse_kernel::FUSE_AUTO_INVAL_DATA: auto_inval_data,
	fuse_kernel::FUSE_DO_READDIRPLUS: do_readdirplus,
	fuse_kernel::FUSE_READDIRPLUS_AUTO: readdirplus_auto,
	fuse_kernel::FUSE_ASYNC_DIO: async_dio,
	fuse_kernel::FUSE_WRITEBACK_CACHE: writeback_cache,
	fuse_kernel::FUSE_NO_OPEN_SUPPORT: no_open_support,
	fuse_kernel::FUSE_PARALLEL_DIROPS: parallel_dirops,
	fuse_kernel::FUSE_HANDLE_KILLPRIV: handle_killpriv,
	fuse_kernel::FUSE_POSIX_ACL: posix_acl,
	fuse_kernel::FUSE_ABORT_ERROR: abort_error,
}

// }}}
