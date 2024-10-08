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

use core::convert;
use core::fmt;

use crate::internal::debug;

/// The maximum length of an extended attribute name list, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on extended attribute names.
///
/// | Platform | Symbolic constant in `limits.h` | XattrValue       |
/// |----------|---------------------------------|-------------|
/// | FreeBSD  |                                 | (unlimited) |
/// | Linux    | `XATTR_LIST_MAX`                | 65536       |
///
/// Note that even if the platform imposes no limit, there is still an implicit
/// limit of approximately [`u32::MAX`] implied by the FUSE protocol.
pub const XATTR_LIST_MAX: Option<usize> = xattr_list_max();

const fn xattr_list_max() -> Option<usize> {
	if cfg!(target_os = "linux") {
		Some(65536) // XATTR_LIST_MAX
	} else {
		None
	}
}

/// The maximum length of an extended attribute name, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on extended attribute names.
///
/// | Platform | Symbolic constant in `limits.h` | Value |
/// |----------|---------------------------------|-------|
/// | FreeBSD  | `EXTATTR_MAXNAMELEN`            | 255   |
/// | Linux    | `XATTR_NAME_MAX`                | 255   |
pub const XATTR_NAME_MAX: Option<usize> = xattr_name_max();

const fn xattr_name_max() -> Option<usize> {
	if cfg!(target_os = "freebsd") {
		Some(255) // EXTATTR_MAXNAMELEN
	} else if cfg!(target_os = "linux") {
		Some(255) // XATTR_NAME_MAX
	} else {
		None
	}
}

/// The maximum length of an extended attribute value, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on extended attribute names.
///
/// | Platform | Symbolic constant in `limits.h` | XattrValue       |
/// |----------|---------------------------------|-------------|
/// | FreeBSD  |                                 | (unlimited) |
/// | Linux    | `XATTR_SIZE_MAX`                | 65536       |
///
/// Note that even if the platform imposes no limit on the maximum length
/// of an extended attribute value, there is still an implicit limit of
/// approximately [`u32::MAX`] implied by the FUSE protocol.
pub const XATTR_SIZE_MAX: Option<usize> = xattr_size_max();

const fn xattr_size_max() -> Option<usize> {
	if cfg!(target_os = "linux") {
		Some(65536) // XATTR_SIZE_MAX
	} else {
		None
	}
}

/// Errors that may occur when validating an extended attribute name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum XattrNameError {
	/// The input is empty.
	Empty,
	/// The input contains `NUL`.
	ContainsNul,
	/// The input length in bytes exceeds [`XATTR_NAME_MAX`].
	ExceedsMaxLen,
}

/// A borrowed extended attribute name.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the name of an extended attribute. It can be constructed safely from a
/// `&str` or `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is non-empty, is less than [`XATTR_NAME_MAX`] bytes in length, and
/// does not contain `NUL`.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XattrName {
	bytes: [u8],
}

impl XattrName {
	/// Attempts to reborrow a string as an extended attribute name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// [`XATTR_NAME_MAX`] bytes, or contains `NUL`.
	#[inline]
	pub fn new(name: &str) -> Result<&XattrName, XattrNameError> {
		Self::from_bytes(name.as_bytes())
	}

	/// Reborrows a string as an extended attribute name, without validation.
	///
	/// # Safety
	///
	/// The provided string must be non-empty, must be no longer than
	/// [`XATTR_NAME_MAX`] bytes, and must not contain `NUL`.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(name: &str) -> &XattrName {
		Self::from_bytes_unchecked(name.as_bytes())
	}

	/// Attempts to reborrow a byte slice as an extended attribute name.
	///
	/// # Errors
	///
	/// Returns an error if the slice is empty, is longer than [`XATTR_NAME_MAX`]
	/// bytes, or contains `NUL`.
	#[inline]
	pub fn from_bytes(bytes: &[u8]) -> Result<&XattrName, XattrNameError> {
		if bytes.is_empty() {
			return Err(XattrNameError::Empty);
		}
		if let Some(max_len) = XATTR_NAME_MAX {
			if bytes.len() > max_len {
				return Err(XattrNameError::ExceedsMaxLen);
			}
		}
		if bytes.contains(&0) {
			return Err(XattrNameError::ContainsNul);
		}
		Ok(unsafe { Self::from_bytes_unchecked(bytes) })
	}

	/// Reborrows a byte slice as an extended attribute name, without
	/// validation.
	///
	/// # Safety
	///
	/// The provided `&[u8]` must be non-empty, must be no longer than
	/// [`XATTR_NAME_MAX`] bytes, and must not contain `NUL`.
	#[inline]
	#[must_use]
	pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &XattrName {
		&*(bytes as *const [u8] as *const XattrName)
	}

	/// Returns how many bytes are required to encode this name.
	#[inline]
	#[must_use]
	pub fn size(&self) -> usize {
		self.bytes.len().saturating_add(1)
	}

	/// Converts this `XattrName` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Attempts to convert this `XattrName` to a `&str`.
	///
	/// # Errors
	///
	/// Returns an error if the name is not UTF-8.
	#[inline]
	pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.bytes)
	}
}

impl fmt::Debug for XattrName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::bytes(&self.bytes).fmt(fmt)
	}
}

impl<'a> convert::TryFrom<&'a XattrName> for &'a str {
	type Error = core::str::Utf8Error;
	fn try_from(name: &'a XattrName) -> Result<&'a str, core::str::Utf8Error> {
		name.as_str()
	}
}

impl<'a> convert::TryFrom<&'a str> for &'a XattrName {
	type Error = XattrNameError;
	fn try_from(name: &'a str) -> Result<&'a XattrName, XattrNameError> {
		XattrName::new(name)
	}
}

impl PartialEq<str> for XattrName {
	fn eq(&self, other: &str) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for XattrName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<XattrName> for str {
	fn eq(&self, other: &XattrName) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<XattrName> for [u8] {
	fn eq(&self, other: &XattrName) -> bool {
		self.eq(other.as_bytes())
	}
}

/// Errors that may occur when validating an extended attribute value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum XattrValueError {
	/// The input length in bytes exceeds [`XATTR_SIZE_MAX`].
	ExceedsMaxLen,
}

/// A borrowed extended attribute value.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the value of an extended attribute. It can be constructed safely from a
/// `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is less than [`XATTR_SIZE_MAX`] bytes in length.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XattrValue {
	bytes: [u8],
}

impl XattrValue {
	/// Attempts to reborrow a byte slice as an extended attribute value.
	///
	/// # Errors
	///
	/// Returns an error if the slice is longer than [`XATTR_SIZE_MAX`] bytes.
	#[inline]
	pub const fn new(bytes: &[u8]) -> Result<&XattrValue, XattrValueError> {
		if let Some(max_len) = XATTR_SIZE_MAX {
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
	/// The provided `&[u8]` must be no longer than [`XATTR_SIZE_MAX`] bytes.
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
