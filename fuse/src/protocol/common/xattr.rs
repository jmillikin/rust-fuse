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

use core::{cmp, fmt};

use crate::internal::fuse_io;

#[derive(Hash)]
#[repr(transparent)]
pub struct XattrName([u8]);

#[rustfmt::skip]
pub const XATTR_LIST_MAX: usize = {
	#[cfg(target_os = "linux")] { 65536 }
};

#[rustfmt::skip]
pub const XATTR_NAME_MAX: usize = {
	#[cfg(target_os = "linux")] { 255 }
};

#[rustfmt::skip]
pub const XATTR_SIZE_MAX: usize = {
	#[cfg(target_os = "linux")] { 65536 }
};

impl XattrName {
	pub(crate) fn new<'a>(
		bytes: fuse_io::NulTerminatedBytes<'a>,
	) -> &'a XattrName {
		let bytes = bytes.to_bytes_without_nul();
		unsafe { &*(bytes as *const [u8] as *const XattrName) }
	}

	pub fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a XattrName> {
		let len = bytes.len();
		if len == 0 || len > XATTR_NAME_MAX {
			return None;
		}
		if bytes.contains(&0) {
			return None;
		}
		Some(unsafe { &*(bytes as *const [u8] as *const XattrName) })
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl fmt::Debug for XattrName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for XattrName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use core::fmt::Debug;
		super::DebugBytesAsString(&self.0).fmt(fmt)
	}
}

impl Eq for XattrName {}

impl PartialEq for XattrName {
	fn eq(&self, other: &XattrName) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for XattrName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl Ord for XattrName {
	fn cmp(&self, other: &XattrName) -> cmp::Ordering {
		self.as_bytes().cmp(&other.as_bytes())
	}
}

impl PartialEq<XattrName> for [u8] {
	fn eq(&self, other: &XattrName) -> bool {
		self.eq(other.as_bytes())
	}
}

impl PartialOrd for XattrName {
	fn partial_cmp(&self, other: &XattrName) -> Option<cmp::Ordering> {
		self.as_bytes().partial_cmp(&other.as_bytes())
	}
}
