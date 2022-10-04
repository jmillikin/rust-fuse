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

//! Extended attributes.
//!
//! Extended attributes are `(name, value)` pairs associated with filesystem
//! objects. In the context of FUSE, they are often used to expose structured
//! filesystem-specific properties not otherwise supported by the `FUSE_GETATTR`
//! and `FUSE_SETATTR` operations.

use core::fmt;
use core::fmt::Debug;

use crate::protocol::common::DebugBytesAsString;

#[cfg(target_os = "freebsd")]
const EXTATTR_MAXNAMELEN: usize = 255;

#[cfg(target_os = "linux")]
const XATTR_NAME_MAX: usize = 255;

#[cfg(target_os = "linux")]
pub(crate) const XATTR_LIST_MAX: usize = 65536;

#[cfg(target_os = "linux")]
const XATTR_SIZE_MAX: usize = 65536;

/// Errors that may occur when validating an extended attribute name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NameError {
	/// The input is empty.
	Empty,
	/// The input contains `NUL`.
	ContainsNul,
	/// The input length in bytes exceeds [`Name::MAX_LEN`].
	ExceedsMaxLen,
}

/// A borrowed extended attribute name.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the name of an extended attribute. It can be constructed safely from a
/// `&str` or `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is non-empty, is less than [`Name::MAX_LEN`] bytes in length, and
/// does not contain `NUL`.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Name {
	bytes: [u8],
}

#[cfg(target_os = "freebsd")]
macro_rules! xattr_name_max_len {
	() => { Some(EXTATTR_MAXNAMELEN) }
}

#[cfg(target_os = "linux")]
macro_rules! xattr_name_max_len {
	() => { Some(XATTR_NAME_MAX) }
}

impl Name {
	/// The maximum length of an extended attribute name, in bytes.
	///
	/// This value is platform-specific. If `None`, then the platform does not
	/// impose a maximum length on extended attribute names.
	///
	/// | Platform | Symbolic constant in `limits.h` | Value |
	/// |----------|---------------------------------|-------|
	/// | FreeBSD  | `EXTATTR_MAXNAMELEN`            | 255   |
	/// | Linux    | `XATTR_NAME_MAX`                | 255   |
	///
	pub const MAX_LEN: Option<usize> = xattr_name_max_len!();

	/// Attempts to reborrow a string as an extended attribute name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// [`Name::MAX_LEN`] bytes, or contains `NUL`.
	#[inline]
	pub fn new(name: &str) -> Result<&Name, NameError> {
		Self::from_bytes(name.as_bytes())
	}

	/// Reborrows a string as an extended attribute name, without validation.
	///
	/// # Safety
	///
	/// The provided string must be non-empty, must be no longer than
	/// [`Name::MAX_LEN`] bytes, and must not contain `NUL`.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(name: &str) -> &Name {
		Self::from_bytes_unchecked(name.as_bytes())
	}

	/// Attempts to reborrow a byte slice as an extended attribute name.
	///
	/// # Errors
	///
	/// Returns an error if the slice is empty, is longer than [`Name::MAX_LEN`]
	/// bytes, or contains `NUL`.
	#[inline]
	pub fn from_bytes(bytes: &[u8]) -> Result<&Name, NameError> {
		if bytes.is_empty() {
			return Err(NameError::Empty);
		}
		if let Some(max_len) = Name::MAX_LEN {
			if bytes.len() > max_len {
				return Err(NameError::ExceedsMaxLen);
			}
		}
		if bytes.contains(&0) {
			return Err(NameError::ContainsNul);
		}
		Ok(unsafe { Self::from_bytes_unchecked(bytes) })
	}

	/// Reborrows a byte slice as an extended attribute name, without
	/// validation.
	///
	/// # Safety
	///
	/// The provided `&[u8]` must be non-empty, must be no longer than
	/// [`Name::MAX_LEN`] bytes, and must not contain `NUL`.
	#[inline]
	#[must_use]
	pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Name {
		&*(bytes as *const [u8] as *const Name)
	}

	/// Returns how many bytes are required to encode this name.
	#[inline]
	#[must_use]
	pub fn size(&self) -> usize {
		self.bytes.len().saturating_add(1)
	}

	/// Converts this `Name` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Attempts to convert this `Name` to a `&str`.
	///
	/// # Errors
	///
	/// Returns an error if the name is not UTF-8.
	#[inline]
	pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.bytes)
	}
}

impl Debug for Name {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		DebugBytesAsString(&self.bytes).fmt(fmt)
	}
}

impl PartialEq<str> for Name {
	fn eq(&self, other: &str) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for Name {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<Name> for str {
	fn eq(&self, other: &Name) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<Name> for [u8] {
	fn eq(&self, other: &Name) -> bool {
		self.eq(other.as_bytes())
	}
}

/// Errors that may occur when validating an extended attribute value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ValueError {
	/// The input length in bytes exceeds [`Value::MAX_LEN`].
	ExceedsMaxLen,
}

/// A borrowed extended attribute value.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the value of an extended attribute. It can be constructed safely from a
/// `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is less than [`Value::MAX_LEN`] bytes in length.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Value {
	bytes: [u8],
}

#[cfg(target_os = "freebsd")]
macro_rules! xattr_value_max_len {
	() => { None }
}

#[cfg(target_os = "linux")]
macro_rules! xattr_value_max_len {
	() => { Some(XATTR_SIZE_MAX) }
}

impl Value {
	/// The maximum length of an extended attribute value, in bytes.
	///
	/// This value is platform-specific. If `None`, then the platform does not
	/// impose a maximum length on extended attribute names.
	///
	/// | Platform | Symbolic constant in `limits.h` | Value       |
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
	/// Returns an error if the slice is longer than [`Value::MAX_LEN`] bytes.
	#[inline]
	pub const fn new(bytes: &[u8]) -> Result<&Value, ValueError> {
		if let Some(max_len) = Value::MAX_LEN {
			if bytes.len() > max_len {
				return Err(ValueError::ExceedsMaxLen);
			}
		}
		Ok(unsafe { Self::new_unchecked(bytes) })
	}

	/// Reborrows a byte slice as an extended attribute value, without
	/// validation.
	///
	/// # Safety
	///
	/// The provided `&[u8]` must be no longer than [`Value::MAX_LEN`] bytes.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(bytes: &[u8]) -> &Value {
		&*(bytes as *const [u8] as *const Value)
	}

	/// Returns how many bytes are required to encode this value.
	#[inline]
	#[must_use]
	pub fn size(&self) -> usize {
		self.bytes.len()
	}

	/// Converts this `Value` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}
}

impl Debug for Value {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bytes.fmt(fmt)
	}
}

impl PartialEq<[u8]> for Value {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<Value> for [u8] {
	fn eq(&self, other: &Value) -> bool {
		self.eq(other.as_bytes())
	}
}
