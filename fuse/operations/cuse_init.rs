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

use crate::{
	CuseDeviceName,
	CuseDeviceNumber,
	Version,
};
use crate::kernel;
use crate::server;

// CuseInitRequest {{{

/// Request type for `CUSE_INIT`.
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

try_from_cuse_request!(CuseInitRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::CUSE_INIT)?;

	let raw: &'a kernel::cuse_init_in = dec.next_sized()?;
	Ok(CuseInitRequest {
		phantom: PhantomData,
		version: Version::new(raw.major, raw.minor),
		flags: CuseInitFlags { bits: raw.flags },
	})
});

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
pub struct CuseInitResponse<'a> {
	pub(crate) raw: kernel::cuse_init_out,
	device_name: Option<&'a CuseDeviceName>,
}

impl<'a> CuseInitResponse<'a> {
	#[must_use]
	pub fn new(device_name: &'a CuseDeviceName) -> CuseInitResponse<'a> {
		CuseInitResponse {
			raw: kernel::cuse_init_out::new(),
			device_name: Some(device_name),
		}
	}

	#[must_use]
	pub(crate) fn new_nameless() -> CuseInitResponse<'static> {
		CuseInitResponse {
			raw: kernel::cuse_init_out::new(),
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
	pub fn device_number(&self) -> CuseDeviceNumber {
		CuseDeviceNumber {
			major: self.raw.dev_major,
			minor: self.raw.dev_minor,
		}
	}

	pub fn set_device_number(&mut self, device_number: CuseDeviceNumber) {
		self.raw.dev_major = device_number.major;
		self.raw.dev_minor = device_number.minor;
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

impl server::CuseReply for CuseInitResponse<'_> {
	fn send_to<S: server::CuseSocket>(
		&self,
		reply_sender: server::CuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		if let Some(device_name) = self.device_name.map(|n| n.as_bytes()) {
			return reply_sender.inner.send_4(
				self.raw.as_bytes(),
				b"DEVNAME=",
				device_name,
				b"\x00",
			);
		}
		return reply_sender.inner.send_1(self.raw.as_bytes());
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
	use crate::kernel;
	bitflags!(CuseInitFlag, CuseInitFlags, u32, {
		UNRESTRICTED_IOCTL = kernel::CUSE_UNRESTRICTED_IOCTL;
	});
}

// }}}
