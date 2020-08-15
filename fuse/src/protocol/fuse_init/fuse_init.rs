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

use crate::protocol::prelude::*;

#[cfg(test)]
mod fuse_init_test;

// FuseInitRequest {{{

/// Request type for [`FuseHandlers::fuse_init`].
///
/// [`FuseHandlers::fuse_init`]: ../trait.FuseHandlers.html#method.fuse_init
pub struct FuseInitRequest<'a> {
	phantom: PhantomData<&'a ()>,
	version: crate::ProtocolVersion,
	max_readahead: u32,
	flags: FuseInitFlags,
}

impl FuseInitRequest<'_> {
	pub fn version(&self) -> crate::ProtocolVersion {
		self.version
	}

	pub fn max_readahead(&self) -> u32 {
		self.max_readahead
	}

	pub fn set_max_readahead(&mut self, max_readahead: u32) {
		self.max_readahead = max_readahead;
	}

	pub fn flags(&self) -> FuseInitFlags {
		self.flags
	}

	pub fn set_flags(&mut self, flags: FuseInitFlags) {
		self.flags = flags;
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
	) -> io::Result<Self> {
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
				version: crate::ProtocolVersion::new(
					raw_v7p1.major,
					raw_v7p1.minor,
				),
				max_readahead: 0,
				flags: FuseInitFlags::from_bits(0),
			});
		}

		let raw: &'a fuse_kernel::fuse_init_in = dec.next_sized()?;
		Ok(FuseInitRequest {
			phantom: PhantomData,
			version: crate::ProtocolVersion::new(raw.major, raw.minor),
			max_readahead: raw.max_readahead,
			flags: FuseInitFlags::from_bits(raw.flags),
		})
	}
}

// }}}

// FuseInitResponse {{{

/// Response type for [`FuseHandlers::fuse_init`].
///
/// [`FuseHandlers::fuse_init`]: ../trait.FuseHandlers.html#method.fuse_init
pub struct FuseInitResponse {
	raw: fuse_kernel::fuse_init_out,
}

impl FuseInitResponse {
	pub fn new(version: crate::ProtocolVersion) -> FuseInitResponse {
		Self {
			raw: fuse_kernel::fuse_init_out {
				major: version.major(),
				minor: version.minor(),
				max_readahead: 0,
				flags: 0,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
				time_gran: 0,
				unused: [0; 9],
			},
		}
	}

	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn for_request(request: &FuseInitRequest) -> FuseInitResponse {
		Self::for_request_impl(request)
	}

	pub(crate) fn for_request_impl(request: &FuseInitRequest) -> Self {
		let version = request.version();

		let v_minor;
		if version.major() == fuse_kernel::FUSE_KERNEL_VERSION {
			// Use the kernel's minor version, unless it's too new for this
			// library in which case use ours.
			v_minor =
				min(version.minor(), fuse_kernel::FUSE_KERNEL_MINOR_VERSION);
		} else {
			// See comment in `FuseInitRequest::decode_request()`. Major version
			// mismatch results in a dummy `FuseInitResponse`. We set our best
			// minor version here as a hint to the kernel.
			v_minor = fuse_kernel::FUSE_KERNEL_MINOR_VERSION;
		}

		let v_major = fuse_kernel::FUSE_KERNEL_VERSION;
		let version = crate::ProtocolVersion::new(v_major, v_minor);
		let mut response = FuseInitResponse::new(version);
		response.set_max_readahead(request.max_readahead());

		let mut flags = request.flags();
		flags.bits = 0; // clear unknown flag bits
		flags.do_readdirplus = false;
		flags.readdirplus_auto = false;

		response.set_flags(flags);
		response
	}

	pub fn version(&self) -> crate::ProtocolVersion {
		crate::ProtocolVersion::new(self.raw.major, self.raw.minor)
	}

	pub fn max_readahead(&self) -> u32 {
		self.raw.max_readahead
	}

	pub fn set_max_readahead(&mut self, max_readahead: u32) {
		self.raw.max_readahead = max_readahead;
	}

	pub fn flags(&self) -> FuseInitFlags {
		FuseInitFlags::from_bits(self.raw.flags)
	}

	pub fn set_flags(&mut self, flags: FuseInitFlags) {
		self.raw.flags = flags.to_bits();
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
			.field("version", &self.version())
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

impl fuse_io::EncodeResponse for FuseInitResponse {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		if self.raw.minor >= 23 {
			return enc.encode_sized(&self.raw);
		}

		if self.raw.minor >= 5 {
			let compat: &'a fuse_init_out_v7p5 =
				unsafe { std::mem::transmute(&self.raw) };
			return enc.encode_sized(compat);
		}

		let compat: &'a fuse_init_out_v7p1 =
			unsafe { std::mem::transmute(&self.raw) };
		enc.encode_sized(compat)
	}
}

// }}}

// FuseInitFlags {{{

bitflags_struct! {
	pub struct FuseInitFlags(u32);

	FUSE_ASYNC_READ: async_read,
	FUSE_POSIX_LOCKS: posix_locks,
	FUSE_FILE_OPS: file_ops,
	FUSE_ATOMIC_O_TRUNC: atomic_o_trunc,
	FUSE_EXPORT_SUPPORT: export_support,
	FUSE_BIG_WRITES: big_writes,
	FUSE_DONT_MASK: dont_mask,
	FUSE_SPLICE_WRITE: splice_write,
	FUSE_SPLICE_MOVE: splice_move,
	FUSE_SPLICE_READ: splice_read,
	FUSE_FLOCK_LOCKS: flock_locks,
	FUSE_HAS_IOCTL_DIR: has_ioctl_dir,
	FUSE_AUTO_INVAL_DATA: auto_inval_data,
	FUSE_DO_READDIRPLUS: do_readdirplus,
	FUSE_READDIRPLUS_AUTO: readdirplus_auto,
	FUSE_ASYNC_DIO: async_dio,
	FUSE_WRITEBACK_CACHE: writeback_cache,
	FUSE_NO_OPEN_SUPPORT: no_open_support,
	FUSE_PARALLEL_DIROPS: parallel_dirops,
	FUSE_HANDLE_KILLPRIV: handle_killpriv,
	FUSE_POSIX_ACL: posix_acl,
	FUSE_ABORT_ERROR: abort_error,
}

// }}}
