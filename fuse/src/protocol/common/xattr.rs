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

use crate::io::decode;

// XattrError {{{

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct XattrError {
	kind: XattrErrorKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum XattrErrorKind {
	ExceedsListMax(usize),
	ExceedsCapacity(usize, usize),
	ExceedsSizeMax(usize),
	ExceedsRequestSize(usize, u32),
	NameEmpty,
	ExceedsNameMax(usize),
	NameContainsNul,
}

impl XattrError {
	pub(crate) fn exceeds_list_max(response_size: usize) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsListMax(response_size),
		}
	}

	pub(crate) fn exceeds_capacity(
		response_size: usize,
		capacity: usize,
	) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsCapacity(response_size, capacity),
		}
	}

	pub(crate) fn exceeds_size_max(value_size: usize) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsSizeMax(value_size),
		}
	}

	pub(crate) fn exceeds_request_size(
		response_size: usize,
		request_size: u32,
	) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsRequestSize(
				response_size,
				request_size,
			),
		}
	}

	pub(crate) fn name_empty() -> XattrError {
		XattrError {
			kind: XattrErrorKind::NameEmpty,
		}
	}

	pub(crate) fn exceeds_name_max(name_size: usize) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsNameMax(name_size),
		}
	}

	pub(crate) fn name_contains_nul() -> XattrError {
		XattrError {
			kind: XattrErrorKind::NameContainsNul,
		}
	}
}

impl fmt::Display for XattrError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		fmt::Debug::fmt(self, fmt)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for XattrError {}

// }}}

#[derive(Hash)]
#[repr(transparent)]
pub struct XattrName([u8]);

pub const XATTR_LIST_MAX: usize = {
	#[cfg(target_os = "linux")] { 65536 }
	#[cfg(target_os = "freebsd")] { 65536 }
};

pub const XATTR_NAME_MAX: usize = {
	#[cfg(target_os = "linux")] { 255 }
	#[cfg(target_os = "freebsd")] { 255 }
};

pub const XATTR_SIZE_MAX: usize = {
	#[cfg(target_os = "linux")] { 65536 }
	#[cfg(target_os = "freebsd")] { 65536 }
};

impl XattrName {
	pub(crate) fn new<'a>(
		bytes: decode::NulTerminatedBytes<'a>,
	) -> &'a XattrName {
		Self::new_unchecked(bytes.to_bytes_without_nul())
	}

	pub(crate) fn new_unchecked<'a>(bytes: &'a [u8]) -> &'a XattrName {
		unsafe { &*(bytes as *const [u8] as *const XattrName) }
	}

	pub fn from_bytes<'a>(
		bytes: &'a [u8],
	) -> Result<&'a XattrName, XattrError> {
		let len = bytes.len();
		if len == 0 {
			return Err(XattrError::name_empty());
		}
		if len > XATTR_NAME_MAX {
			return Err(XattrError::exceeds_name_max(len));
		}
		if bytes.contains(&0) {
			return Err(XattrError::name_contains_nul());
		}
		Ok(unsafe { &*(bytes as *const [u8] as *const XattrName) })
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
