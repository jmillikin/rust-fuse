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
mod cuse_init_test;

// CuseInitRequest {{{

/// Request type for [`CuseHandlers::cuse_init`].
///
/// [`CuseHandlers::cuse_init`]: ../../trait.CuseHandlers.html#method.cuse_init
pub struct CuseInitRequest<'a> {
	phantom: PhantomData<&'a ()>,
	version: crate::ProtocolVersion,
	flags: CuseInitFlags,
}

impl CuseInitRequest<'_> {
	pub fn version(&self) -> crate::ProtocolVersion {
		self.version
	}

	pub fn flags(&self) -> &CuseInitFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut CuseInitFlags {
		&mut self.flags
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

impl<'a> fuse_io::DecodeRequest<'a> for CuseInitRequest<'_> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		debug_assert!(dec.header().opcode == fuse_kernel::CUSE_INIT);

		let raw: &'a fuse_kernel::cuse_init_in = dec.next_sized()?;
		Ok(CuseInitRequest {
			phantom: PhantomData,
			version: crate::ProtocolVersion::new(raw.major, raw.minor),
			flags: CuseInitFlags::from_bits(raw.flags),
		})
	}
}

// }}}

// CuseInitResponse {{{

/// Response type for [`CuseHandlers::cuse_init`].
///
/// [`CuseHandlers::cuse_init`]: ../trait.CuseHandlers.html#method.cuse_init
pub struct CuseInitResponse {
	raw: fuse_kernel::cuse_init_out,
	flags: CuseInitFlags,
}

impl CuseInitResponse {
	pub fn new(version: crate::ProtocolVersion) -> CuseInitResponse {
		Self {
			raw: fuse_kernel::cuse_init_out {
				major: version.major(),
				minor: version.minor(),
				unused: 0,
				flags: 0,
				max_read: 0,
				max_write: 0,
				dev_major: 0,
				dev_minor: 0,
				spare: [0; 10],
			},
			flags: CuseInitFlags::new(),
		}
	}

	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn for_request(request: &CuseInitRequest) -> CuseInitResponse {
		Self::for_request_impl(request)
	}

	pub(crate) fn for_request_impl(request: &CuseInitRequest) -> Self {
		let version = request.version();

		let v_minor;
		if version.major() == fuse_kernel::FUSE_KERNEL_VERSION {
			// Use the kernel's minor version, unless it's too new for this
			// library in which case use ours.
			v_minor =
				min(version.minor(), fuse_kernel::FUSE_KERNEL_MINOR_VERSION);
		} else {
			// See comment in `FuseInitRequest::decode_request()`. Major version
			// mismatch results in a dummy `CuseInitResponse`. We set our best
			// minor version here as a hint to the kernel.
			v_minor = fuse_kernel::FUSE_KERNEL_MINOR_VERSION;
		}

		let v_major = fuse_kernel::FUSE_KERNEL_VERSION;
		let version = crate::ProtocolVersion::new(v_major, v_minor);
		let mut response = CuseInitResponse::new(version);

		let mut flags = *request.flags();
		flags.bits = 0; // clear unknown flag bits

		*response.flags_mut() = flags;
		response
	}

	pub fn version(&self) -> crate::ProtocolVersion {
		crate::ProtocolVersion::new(self.raw.major, self.raw.minor)
	}

	pub fn flags(&self) -> &CuseInitFlags {
		&self.flags
	}

	pub fn flags_mut(&mut self) -> &mut CuseInitFlags {
		&mut self.flags
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
}

impl fmt::Debug for CuseInitResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CuseInitResponse")
			.field("version", &self.version())
			.field("flags", self.flags())
			.field("max_read", &self.max_read())
			.field("max_write", &self.max_write())
			.field("dev_major", &self.dev_major())
			.field("dev_minor", &self.dev_minor())
			.finish()
	}
}

impl fuse_io::EncodeResponse for CuseInitResponse {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		let mut out = self.raw;
		out.flags = self.flags.to_bits();
		enc.encode_sized(&out)
	}
}

// }}}

// CuseInitFlags {{{

bitflags_struct! {
	pub struct CuseInitFlags(u32);

	fuse_kernel::CUSE_UNRESTRICTED_IOCTL: unrestricted_ioctl,
}

// }}}
