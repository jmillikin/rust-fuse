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

use crate::server::io::decode;

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct NodeName([u8]);

pub const NODE_NAME_MAX: usize = {
	#[cfg(target_os = "linux")]   { 255 }
	#[cfg(target_os = "freebsd")] { 255 }
};

impl NodeName {
	pub(crate) fn new<'a>(
		bytes: decode::NulTerminatedBytes<'a>,
	) -> &'a NodeName {
		let bytes = bytes.to_bytes_without_nul();
		unsafe { &*(bytes as *const [u8] as *const NodeName) }
	}

	pub fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a NodeName> {
		let len = bytes.len();
		if len == 0 || len > NODE_NAME_MAX {
			return None;
		}
		if bytes.contains(&0) || bytes.contains(&b'/') {
			return None;
		}
		Some(unsafe { Self::new_unchecked(bytes) })
	}

	pub(crate) unsafe fn new_unchecked<'a>(bytes: &'a [u8]) -> &'a NodeName {
		&*(bytes as *const [u8] as *const NodeName)
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl fmt::Debug for NodeName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for NodeName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use core::fmt::Debug;
		super::DebugBytesAsString(&self.0).fmt(fmt)
	}
}

impl PartialEq<[u8]> for NodeName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<NodeName> for [u8] {
	fn eq(&self, other: &NodeName) -> bool {
		self.eq(other.as_bytes())
	}
}

impl PartialEq<str> for NodeName {
	fn eq(&self, other: &str) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<NodeName> for str{
	fn eq(&self, other: &NodeName) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}
