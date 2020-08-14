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
mod setattr_test;

// SetattrRequest {{{

pub struct SetattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_setattr_in,
}

impl SetattrRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	fn get<T>(&self, bitmask: u32, value: T) -> Option<T> {
		if self.raw.valid & bitmask == 0 {
			return None;
		}
		Some(value)
	}

	fn get_timestamp(
		&self,
		bitmask: u32,
		seconds: u64,
		nanos: u32,
	) -> Option<time::SystemTime> {
		if self.raw.valid & bitmask == 0 {
			return None;
		}
		Some(systime(seconds, nanos))
	}

	pub fn handle(&self) -> Option<u64> {
		self.get(fuse_kernel::FATTR_FH, self.raw.fh)
	}

	pub fn size(&self) -> Option<u64> {
		self.get(fuse_kernel::FATTR_SIZE, self.raw.size)
	}

	pub fn lock_owner(&self) -> Option<u64> {
		self.get(fuse_kernel::FATTR_LOCKOWNER, self.raw.lock_owner)
	}

	pub fn atime(&self) -> Option<time::SystemTime> {
		self.get_timestamp(
			fuse_kernel::FATTR_ATIME,
			self.raw.atime,
			self.raw.atimensec,
		)
	}

	pub fn atime_now(&self) -> bool {
		self.raw.valid & fuse_kernel::FATTR_ATIME_NOW > 0
	}

	pub fn mtime(&self) -> Option<time::SystemTime> {
		self.get_timestamp(
			fuse_kernel::FATTR_MTIME,
			self.raw.mtime,
			self.raw.mtimensec,
		)
	}

	pub fn mtime_now(&self) -> bool {
		self.raw.valid & fuse_kernel::FATTR_MTIME_NOW > 0
	}

	pub fn ctime(&self) -> Option<time::SystemTime> {
		self.get_timestamp(
			fuse_kernel::FATTR_CTIME,
			self.raw.ctime,
			self.raw.ctimensec,
		)
	}

	pub fn mode(&self) -> Option<u32> {
		self.get(fuse_kernel::FATTR_MODE, self.raw.mode)
	}

	pub fn user_id(&self) -> Option<u32> {
		self.get(fuse_kernel::FATTR_UID, self.raw.uid)
	}

	pub fn group_id(&self) -> Option<u32> {
		self.get(fuse_kernel::FATTR_GID, self.raw.gid)
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for SetattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_SETATTR);
		let raw = dec.next_sized()?;
		Ok(Self { header, raw })
	}
}

fn systime(seconds: u64, nanos: u32) -> time::SystemTime {
	time::UNIX_EPOCH + time::Duration::new(seconds, nanos)
}

// }}}

// SetattrResponse {{{

pub struct SetattrResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_attr_out,
}

impl SetattrResponse<'_> {
	// TODO: fix API
	pub fn new(request: &SetattrRequest) -> Self {
		SetattrResponse {
			phantom: PhantomData,
			raw: fuse_kernel::fuse_attr_out {
				attr_valid: 0,
				attr_valid_nsec: 0,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: request.header.nodeid,
					..Default::default()
				},
			},
		}
	}

	pub fn attr(&self) -> &NodeAttr {
		NodeAttr::new_ref(&self.raw.attr)
	}

	pub fn attr_mut(&mut self) -> &mut NodeAttr {
		NodeAttr::new_ref_mut(&mut self.raw.attr)
	}

	pub fn cache_duration(&self) -> time::Duration {
		time::Duration::new(self.raw.attr_valid, self.raw.attr_valid_nsec)
	}

	pub fn set_cache_duration(&mut self, cache_duration: time::Duration) {
		self.raw.attr_valid = cache_duration.as_secs();
		self.raw.attr_valid_nsec = cache_duration.subsec_nanos();
	}
}

impl fmt::Debug for SetattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetattrResponse")
			.field("attr", self.attr())
			.field("cache_duration", &self.cache_duration())
			.finish()
	}
}

impl fuse_io::EncodeResponse for SetattrResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		// The `fuse_attr::blksize` field was added in FUSE v7.9.
		if enc.version().minor() < 9 {
			let buf: &[u8] = unsafe {
				slice::from_raw_parts(
					(&self.raw as *const fuse_kernel::fuse_attr_out)
						as *const u8,
					fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE,
				)
			};
			return enc.encode_bytes(buf);
		}

		enc.encode_sized(&self.raw)
	}
}

// }}}
