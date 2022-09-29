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

//! Implements the `CUSE_INIT` operation.

use core::fmt;
use core::marker::PhantomData;
use core::mem::size_of;
use core::slice;

use crate::Version;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::encode;

use crate::protocol::common::DebugBytesAsString;

// CuseDeviceName {{{

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CuseDeviceName([u8]);

impl CuseDeviceName {
	pub fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a CuseDeviceName> {
		if bytes.is_empty() || bytes.contains(&0) {
			return None;
		}
		Some(unsafe { &*(bytes as *const [u8] as *const CuseDeviceName) })
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl fmt::Debug for CuseDeviceName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for CuseDeviceName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use core::fmt::Debug;
		DebugBytesAsString(&self.0).fmt(fmt)
	}
}

impl PartialEq<[u8]> for CuseDeviceName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<CuseDeviceName> for [u8] {
	fn eq(&self, other: &CuseDeviceName) -> bool {
		self.eq(other.as_bytes())
	}
}

// }}}

// CuseInitRequest {{{

/// Request type for `CUSE_INIT`.
///
/// See the [module-level documentation](self) for an overview of the
/// `CUSE_INIT` operation.
pub struct CuseInitRequest<'a> {
	phantom: PhantomData<&'a ()>,
	version: Version,
	flags: CuseInitFlags,
}

impl<'a> CuseInitRequest<'a> {
	pub fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::CUSE_INIT)?;

		let raw: &'a fuse_kernel::cuse_init_in = dec.next_sized()?;
		Ok(CuseInitRequest {
			phantom: PhantomData,
			version: Version::new(raw.major, raw.minor),
			flags: CuseInitFlags { bits: raw.flags },
		})
	}

	pub fn version(&self) -> Version {
		self.version
	}

	pub fn flags(&self) -> CuseInitFlags {
		self.flags
	}

	pub fn mut_flags(&mut self) -> &mut CuseInitFlags {
		&mut self.flags
	}

	pub fn set_flags(&mut self, flags: CuseInitFlags) {
		self.flags = flags;
	}
}

impl fmt::Debug for CuseInitRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CuseInitRequest")
			.field("version", &self.version)
			.field("flags", &self.flags)
			.finish()
	}
}

// }}}

// CuseInitResponse {{{

/// Response type for `CUSE_INIT`.
///
/// See the [module-level documentation](self) for an overview of the
/// `CUSE_INIT` operation.
pub struct CuseInitResponse<'a> {
	raw: fuse_kernel::cuse_init_out,
	flags: CuseInitFlags,
	device_name: Option<&'a CuseDeviceName>,
}

impl<'a> CuseInitResponse<'a> {
	pub fn new(device_name: &'a CuseDeviceName) -> CuseInitResponse<'a> {
		CuseInitResponse {
			raw: fuse_kernel::cuse_init_out::zeroed(),
			flags: CuseInitFlags::new(),
			device_name: Some(device_name),
		}
	}

	pub(crate) fn new_nameless() -> CuseInitResponse<'static> {
		CuseInitResponse {
			raw: fuse_kernel::cuse_init_out::zeroed(),
			flags: CuseInitFlags::new(),
			device_name: None,
		}
	}

	pub fn version(&self) -> Version {
		Version::new(self.raw.major, self.raw.minor)
	}

	pub fn set_version(&mut self, v: Version) {
		self.raw.major = v.major();
		self.raw.minor = v.minor();
	}

	pub fn flags(&self) -> CuseInitFlags {
		self.flags
	}

	pub fn mut_flags(&mut self) -> &mut CuseInitFlags {
		&mut self.flags
	}

	pub fn set_flags(&mut self, flags: CuseInitFlags) {
		self.flags = flags;
	}

	pub fn max_read(&self) -> u32 {
		self.raw.max_read
	}

	pub fn set_max_read(&mut self, max_read: u32) {
		self.raw.max_read = max_read;
	}

	pub fn max_write(&self) -> u32 {
		self.raw.max_write
	}

	pub fn set_max_write(&mut self, max_write: u32) {
		self.raw.max_write = max_write;
	}

	pub fn dev_major(&self) -> u32 {
		self.raw.dev_major
	}

	pub fn set_dev_major(&mut self, dev_major: u32) {
		self.raw.dev_major = dev_major;
	}

	pub fn dev_minor(&self) -> u32 {
		self.raw.dev_minor
	}

	pub fn set_dev_minor(&mut self, dev_minor: u32) {
		self.raw.dev_minor = dev_minor;
	}

	pub fn set_device_number(&mut self, major: u32, minor: u32) {
		self.raw.dev_major = major;
		self.raw.dev_minor = minor;
	}
}

response_send_funcs!(CuseInitResponse<'_>);

impl fmt::Debug for CuseInitResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut dbg = fmt.debug_struct("CuseInitResponse");
		if let Some(device_name) = self.device_name {
			dbg.field("device_name", &device_name);
		}
		dbg
			.field("flags", &self.flags())
			.field("max_read", &self.max_read())
			.field("max_write", &self.max_write())
			.field("dev_major", &self.dev_major())
			.field("dev_minor", &self.dev_minor())
			.finish()
	}
}

impl CuseInitResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let mut out = self.raw;
		out.flags = self.flags.bits;
		let out_buf: &[u8] = unsafe {
			slice::from_raw_parts(
				(&out as *const fuse_kernel::cuse_init_out) as *const u8,
				size_of::<fuse_kernel::cuse_init_out>(),
			)
		};
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		match self.device_name.map(|n| n.as_bytes()) {
			None => enc.encode_bytes(out_buf),
			Some(device_name) => {
				enc.encode_bytes_4(out_buf, b"DEVNAME=", device_name, b"\x00")
			},
		}
	}
}

// }}}

// CuseInitFlags {{{

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CuseInitFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CuseInitFlag {
	mask: u32,
}

mod flags {
	use crate::internal::fuse_kernel;
	bitflags!(CuseInitFlag, CuseInitFlags, u32, {
		UNRESTRICTED_IOCTL = fuse_kernel::CUSE_UNRESTRICTED_IOCTL;
	});
}

// }}}