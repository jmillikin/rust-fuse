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

use crate::Version;
use crate::cuse;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::encode;

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

impl CuseInitRequest<'_> {
	#[must_use]
	pub fn version(&self) -> Version {
		self.version
	}

	#[must_use]
	pub fn flags(&self) -> CuseInitFlags {
		self.flags
	}
}

impl<'a> CuseInitRequest<'a> {
	pub fn from_request(
		request: server::Request<'a>,
	) -> Result<CuseInitRequest<'a>, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::CUSE_INIT)?;

		let raw: &'a fuse_kernel::cuse_init_in = dec.next_sized()?;
		Ok(CuseInitRequest {
			phantom: PhantomData,
			version: Version::new(raw.major, raw.minor),
			flags: CuseInitFlags { bits: raw.flags },
		})
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
	device_name: Option<&'a cuse::DeviceName>,
}

impl<'a> CuseInitResponse<'a> {
	#[must_use]
	pub fn new(device_name: &'a cuse::DeviceName) -> CuseInitResponse<'a> {
		CuseInitResponse {
			raw: fuse_kernel::cuse_init_out::zeroed(),
			device_name: Some(device_name),
		}
	}

	#[must_use]
	pub(crate) fn new_nameless() -> CuseInitResponse<'static> {
		CuseInitResponse {
			raw: fuse_kernel::cuse_init_out::zeroed(),
			device_name: None,
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
	pub fn flags(&self) -> CuseInitFlags {
		CuseInitFlags { bits: self.raw.flags }
	}

	pub fn set_flags(&mut self, flags: CuseInitFlags) {
		self.raw.flags = flags.bits;
	}

	#[inline]
	pub fn update_flags(&mut self, f: impl FnOnce(&mut CuseInitFlags)) {
		let mut flags = self.flags();
		f(&mut flags);
		self.set_flags(flags)
	}

	#[must_use]
	pub fn max_read(&self) -> u32 {
		self.raw.max_read
	}

	pub fn set_max_read(&mut self, max_read: u32) {
		self.raw.max_read = max_read;
	}

	#[must_use]
	pub fn max_write(&self) -> u32 {
		self.raw.max_write
	}

	pub fn set_max_write(&mut self, max_write: u32) {
		self.raw.max_write = max_write;
	}

	#[must_use]
	pub fn device_number(&self) -> cuse::DeviceNumber {
		cuse::DeviceNumber::new(
			self.raw.dev_major,
			self.raw.dev_minor,
		)
	}

	pub fn set_device_number(&mut self, device_number: cuse::DeviceNumber) {
		self.raw.dev_major = device_number.major();
		self.raw.dev_minor = device_number.minor();
	}
}

impl fmt::Debug for CuseInitResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut dbg = fmt.debug_struct("CuseInitResponse");
		if let Some(device_name) = self.device_name {
			dbg.field("device_name", &device_name);
		}
		dbg
			.field("version", &self.version())
			.field("flags", &self.flags())
			.field("max_read", &self.max_read())
			.field("max_write", &self.max_write())
			.field("device_number", &self.device_number())
			.finish()
	}
}

impl CuseInitResponse<'_> {
	pub fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
	) -> server::Response<'a> {
		match self.device_name.map(|n| n.as_bytes()) {
			None => encode::sized(header, &self.raw),
			Some(device_name) => encode::sized_bytes3(
				header,
				&self.raw,
				b"DEVNAME=",
				device_name,
				b"\x00",
			),
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
