// Copyright 2022 John Millikin and the rust-fuse contributors.
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

/// Errors that may occur when validating an extended attribute value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum XattrValueError {
	/// The input length in bytes exceeds [`XattrValue::MAX_LEN`].
	ExceedsMaxLen,
}

/// A borrowed extended attribute value.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the value of an extended attribute. It can be constructed safely from a
/// `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is less than [`XattrValue::MAX_LEN`] bytes in length.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XattrValue {
	bytes: [u8],
}

#[cfg(target_os = "linux")]
const XATTR_SIZE_MAX: usize = 65536;

#[cfg(target_os = "freebsd")]
macro_rules! xattr_value_max_len {
	() => { None }
}

#[cfg(target_os = "linux")]
macro_rules! xattr_value_max_len {
	() => { Some(XATTR_SIZE_MAX) }
}

impl XattrValue {
	/// The maximum length of an extended attribute value, in bytes.
	///
	/// This value is platform-specific. If `None`, then the platform does not
	/// impose a maximum length on extended attribute names.
	///
	/// | Platform | Symbolic constant in `limits.h` | XattrValue       |
	/// |----------|---------------------------------|-------------|
	/// | Linux    | `XATTR_SIZE_MAX`                | 65536       |
	/// | FreeBSD  |                                 | (unlimited) |
	///
	/// Note that even if the platform imposes no limit on the maximum length
	/// of an extended attribute value, there is still an implicit limit of
	/// approximately [`u32::MAX`] implied by the FUSE wire protocol.
	pub const MAX_LEN: Option<usize> = xattr_value_max_len!();

	/// Attempts to reborrow a byte slice as an extended attribute value.
	///
	/// # Errors
	///
	/// Returns an error if the slice is longer than [`XattrValue::MAX_LEN`] bytes.
	#[inline]
	pub const fn new(bytes: &[u8]) -> Result<&XattrValue, XattrValueError> {
		if let Some(max_len) = XattrValue::MAX_LEN {
			if bytes.len() > max_len {
				return Err(XattrValueError::ExceedsMaxLen);
			}
		}
		Ok(unsafe { Self::new_unchecked(bytes) })
	}

	/// Reborrows a byte slice as an extended attribute value, without
	/// validation.
	///
	/// # Safety
	///
	/// The provided `&[u8]` must be no longer than [`XattrValue::MAX_LEN`] bytes.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(bytes: &[u8]) -> &XattrValue {
		&*(bytes as *const [u8] as *const XattrValue)
	}

	/// Returns how many bytes are required to encode this value.
	#[inline]
	#[must_use]
	pub fn size(&self) -> usize {
		self.bytes.len()
	}

	/// Converts this `XattrValue` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}
}

impl fmt::Debug for XattrValue {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bytes.fmt(fmt)
	}
}

impl PartialEq<[u8]> for XattrValue {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<XattrValue> for [u8] {
	fn eq(&self, other: &XattrValue) -> bool {
		self.eq(other.as_bytes())
	}
}
