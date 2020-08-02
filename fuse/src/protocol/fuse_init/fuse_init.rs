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
#[derive(Debug)]
pub struct FuseInitRequest {
	protocol_version: crate::ProtocolVersion,
	max_readahead: u32,
	flags: FuseInitFlags,
}

impl FuseInitRequest {
	pub fn protocol_version(&self) -> crate::ProtocolVersion {
		self.protocol_version
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

#[repr(C)]
struct fuse_init_in_v7p1 {
	major: u32,
	minor: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for FuseInitRequest {
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
				protocol_version: crate::ProtocolVersion::new(
					raw_v7p1.major,
					raw_v7p1.minor,
				),
				max_readahead: 0,
				flags: FuseInitFlags { bits: 0 },
			});
		}

		let raw: &'a fuse_kernel::fuse_init_in = dec.next_sized()?;
		Ok(FuseInitRequest {
			protocol_version: crate::ProtocolVersion::new(raw.major, raw.minor),
			max_readahead: raw.max_readahead,
			flags: FuseInitFlags { bits: raw.flags },
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
	pub fn new(version: crate::ProtocolVersion) -> Self {
		FuseInitResponse {
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

	pub fn for_request(request: &FuseInitRequest) -> Self {
		let version = request.protocol_version();

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
		// TODO: only set flags that are known to this library.
		response.set_flags(request.flags());
		response
	}

	pub fn protocol_version(&self) -> crate::ProtocolVersion {
		crate::ProtocolVersion::new(self.raw.major, self.raw.minor)
	}

	pub fn max_readahead(&self) -> u32 {
		self.raw.max_readahead
	}

	pub fn set_max_readahead(&mut self, max_readahead: u32) {
		self.raw.max_readahead = max_readahead;
	}

	pub fn flags(&self) -> FuseInitFlags {
		FuseInitFlags {
			bits: self.raw.flags,
		}
	}

	pub fn set_flags(&mut self, flags: FuseInitFlags) {
		self.raw.flags = flags.bits;
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

	pub fn time_gran(&self) -> u32 {
		self.raw.time_gran
	}

	pub fn set_time_gran(&mut self, time_gran: u32) {
		self.raw.time_gran = time_gran;
	}
}

impl fmt::Debug for FuseInitResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FuseInitResponse")
			.field("protocol_version", &self.protocol_version())
			.field("max_readahead", &self.max_readahead())
			.field("flags", &self.flags())
			.field("max_background", &self.max_background())
			.field("congestion_threshold", &self.congestion_threshold())
			.field("max_write", &self.max_write())
			.field("time_gran", &self.time_gran())
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FuseInitFlag {
	bits: u32,
}

impl FuseInitFlag {
	pub const ASYNC_READ: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_ASYNC_READ,
	};
	pub const POSIX_LOCKS: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_POSIX_LOCKS,
	};
	pub const ATOMIC_O_TRUNC: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_ATOMIC_O_TRUNC,
	};
	pub const BIG_WRITES: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_BIG_WRITES,
	};
	pub const EXPORT_SUPPORT: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_EXPORT_SUPPORT,
	};
	pub const DONT_MASK: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_DONT_MASK,
	};
	pub const SPLICE_WRITE: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_SPLICE_WRITE,
	};
	pub const SPLICE_MOVE: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_SPLICE_MOVE,
	};
	pub const SPLICE_READ: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_SPLICE_READ,
	};
	pub const FLOCK_LOCKS: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_FLOCK_LOCKS,
	};
	pub const IOCTL_DIR: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_HAS_IOCTL_DIR,
	};
	pub const AUTO_INVAL_DATA: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_AUTO_INVAL_DATA,
	};
	pub const READDIRPLUS: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_DO_READDIRPLUS,
	};
	pub const READDIRPLUS_AUTO: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_READDIRPLUS_AUTO,
	};
	pub const ASYNC_DIO: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_ASYNC_DIO,
	};
	pub const WRITEBACK_CACHE: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_WRITEBACK_CACHE,
	};
	pub const NO_OPEN_SUPPORT: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_NO_OPEN_SUPPORT,
	};
	pub const PARALLEL_DIROPS: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_PARALLEL_DIROPS,
	};
	pub const HANDLE_KILLPRIV: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_HANDLE_KILLPRIV,
	};
	pub const POSIX_ACL: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_POSIX_ACL,
	};
	pub const ABORT_ERROR: FuseInitFlag = FuseInitFlag {
		bits: fuse_kernel::FUSE_ABORT_ERROR,
	};
}

impl fmt::Binary for FuseInitFlag {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for FuseInitFlag {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for FuseInitFlag {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::Debug for FuseInitFlag {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for FuseInitFlag {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			FuseInitFlag::ASYNC_READ => fmt.write_str("ASYNC_READ"),
			FuseInitFlag::POSIX_LOCKS => fmt.write_str("POSIX_LOCKS"),
			FuseInitFlag::ATOMIC_O_TRUNC => fmt.write_str("ATOMIC_O_TRUNC"),
			FuseInitFlag::BIG_WRITES => fmt.write_str("BIG_WRITES"),
			FuseInitFlag::EXPORT_SUPPORT => fmt.write_str("EXPORT_SUPPORT"),
			FuseInitFlag::DONT_MASK => fmt.write_str("DONT_MASK"),
			FuseInitFlag::SPLICE_WRITE => fmt.write_str("SPLICE_WRITE"),
			FuseInitFlag::SPLICE_MOVE => fmt.write_str("SPLICE_MOVE"),
			FuseInitFlag::SPLICE_READ => fmt.write_str("SPLICE_READ"),
			FuseInitFlag::FLOCK_LOCKS => fmt.write_str("FLOCK_LOCKS"),
			FuseInitFlag::IOCTL_DIR => fmt.write_str("IOCTL_DIR"),
			FuseInitFlag::AUTO_INVAL_DATA => fmt.write_str("AUTO_INVAL_DATA"),
			FuseInitFlag::READDIRPLUS => fmt.write_str("READDIRPLUS"),
			FuseInitFlag::READDIRPLUS_AUTO => fmt.write_str("READDIRPLUS_AUTO"),
			FuseInitFlag::ASYNC_DIO => fmt.write_str("ASYNC_DIO"),
			FuseInitFlag::WRITEBACK_CACHE => fmt.write_str("WRITEBACK_CACHE"),
			FuseInitFlag::NO_OPEN_SUPPORT => fmt.write_str("NO_OPEN_SUPPORT"),
			FuseInitFlag::PARALLEL_DIROPS => fmt.write_str("PARALLEL_DIROPS"),
			FuseInitFlag::HANDLE_KILLPRIV => fmt.write_str("HANDLE_KILLPRIV"),
			FuseInitFlag::POSIX_ACL => fmt.write_str("POSIX_ACL"),
			FuseInitFlag::ABORT_ERROR => fmt.write_str("ABORT_ERROR"),
			_ => write!(fmt, "{:#010X}", self.bits),
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FuseInitFlags {
	bits: u32,
}

impl fmt::Binary for FuseInitFlags {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for FuseInitFlags {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for FuseInitFlags {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::Debug for FuseInitFlags {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for FuseInitFlags {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut chunks = [FuseInitFlag { bits: 0 }; 32];
		let mut ii = 0;
		for bit in 0..31 {
			let mask: u32 = 1 << bit;
			if (self.bits & mask) > 0 {
				chunks[ii].bits = mask;
				ii += 1;
			}
		}
		fmt.debug_list().entries(chunks[0..ii].iter()).finish()
	}
}

impl FuseInitFlags {
	pub fn new() -> FuseInitFlags {
		FuseInitFlags { bits: 0 }
	}

	pub fn get(&self, flag: FuseInitFlag) -> bool {
		(self.bits & flag.bits) > 0
	}
	pub fn set(&mut self, flag: FuseInitFlag, value: bool) {
		if value {
			self.bits |= flag.bits;
		} else {
			self.bits &= !(flag.bits);
		}
	}
}

// }}}
