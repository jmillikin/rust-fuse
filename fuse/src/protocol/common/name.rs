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
pub struct Name {
	bytes: [u8],
}

#[rustfmt::skip]
pub const NAME_MAX: usize = {
	#[cfg(target_os = "linux")]   { 1024 }
	#[cfg(target_os = "freebsd")] {  255 }
};

impl Name {
	pub(crate) fn new<'a>(bytes: fuse_io::NulTerminatedBytes<'a>) -> &'a Name {
		let bytes = bytes.to_bytes_without_nul();
		unsafe { &*(bytes as *const [u8] as *const Name) }
	}

	pub fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a Name> {
		let len = bytes.len();
		if len == 0 || len > NAME_MAX {
			return None;
		}
		if bytes.contains(&b'/') {
			return None;
		}
		Some(unsafe { &*(bytes as *const [u8] as *const Name) })
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}
}

impl fmt::Debug for Name {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for Name {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use core::fmt::Debug;
		super::DebugBytesAsString(&self.bytes).fmt(fmt)
	}
}

impl Eq for Name {}

impl PartialEq for Name {
	fn eq(&self, other: &Name) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for Name {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl Ord for Name {
	fn cmp(&self, other: &Name) -> cmp::Ordering {
		self.as_bytes().cmp(&other.as_bytes())
	}
}

impl PartialEq<Name> for [u8] {
	fn eq(&self, other: &Name) -> bool {
		self.eq(other.as_bytes())
	}
}

impl PartialOrd for Name {
	fn partial_cmp(&self, other: &Name) -> Option<cmp::Ordering> {
		self.as_bytes().partial_cmp(&other.as_bytes())
	}
}
