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

use core::convert;
use core::fmt;

#[cfg(any(doc, feature = "alloc"))]
use alloc::borrow;
#[cfg(any(doc, feature = "alloc"))]
use alloc::boxed;
#[cfg(any(doc, feature = "alloc"))]
use alloc::rc;
#[cfg(any(doc, feature = "alloc"))]
use alloc::string;
#[cfg(any(doc, feature = "alloc"))]
use alloc::vec;
#[cfg(any(doc, feature = "std"))]
use std::sync;

use crate::internal::debug;

#[cfg(target_os = "freebsd")]
const EXTATTR_MAXNAMELEN: usize = 255;

#[cfg(target_os = "linux")]
const XATTR_NAME_MAX: usize = 255;

#[cfg(target_os = "linux")]
pub(crate) const XATTR_LIST_MAX: usize = 65536;

#[cfg(target_os = "linux")]
const XATTR_SIZE_MAX: usize = 65536;

// NOT_FOUND {{{

mod errno {
	#[cfg(target_os = "freebsd")]
	use freebsd_errno as os_errno;

	#[cfg(target_os = "linux")]
	use linux_errno as os_errno;

	use crate::Error;

	#[cfg(target_os = "linux")]
	pub(super) const ENODATA: Error = Error::from_errno(os_errno::ENODATA);

	#[cfg(target_os = "freebsd")]
	pub(super) const ENOATTR: Error = Error::from_errno(os_errno::ENOATTR);
}

#[cfg(target_os = "linux")]
macro_rules! enodata_or_enoattr {
	() => { errno::ENODATA };
}

#[cfg(target_os = "freebsd")]
macro_rules! enodata_or_enoattr {
	() => { errno::ENOATTR };
}

/// The requested extended attribute does not exist.
///
/// This error maps to either `ENODATA` or `ENOATTR`, depending on the
/// target platform.
pub const NOT_FOUND: crate::Error = enodata_or_enoattr!();

// }}}

// NameError {{{

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

// }}}

// Name {{{

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

	/// Converts a `Name` to an owned [`NameBuf`].
	#[cfg(any(doc, feature = "alloc"))]
	#[must_use]
	pub fn to_name_buf(&self) -> NameBuf {
		NameBuf {
			bytes: self.bytes.to_vec(),
		}
	}

	/// Converts a [`Box<Name>`] into a [`NameBuf`] without copying or
	/// allocating.
	///
	/// [`Box<Name>`]: boxed::Box<Name>
	#[cfg(any(doc, feature = "alloc"))]
	#[must_use]
	pub fn into_name_buf(self: boxed::Box<Name>) -> NameBuf {
		let raw = boxed::Box::into_raw(self) as *mut [u8];
		NameBuf {
			bytes: vec::Vec::from(unsafe { boxed::Box::from_raw(raw) }),
		}
	}
}

impl fmt::Debug for Name {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::bytes(&self.bytes).fmt(fmt)
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl borrow::ToOwned for Name {
	type Owned = NameBuf;
	fn to_owned(&self) -> NameBuf {
		self.to_name_buf()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl<'a> From<&'a Name> for borrow::Cow<'a, Name> {
	fn from(name: &'a Name) -> Self {
		borrow::Cow::Borrowed(name)
	}
}

impl<'a> convert::TryFrom<&'a Name> for &'a str {
	type Error = core::str::Utf8Error;
	fn try_from(name: &'a Name) -> Result<&'a str, core::str::Utf8Error> {
		name.as_str()
	}
}

impl<'a> convert::TryFrom<&'a str> for &'a Name {
	type Error = NameError;
	fn try_from(name: &'a str) -> Result<&'a Name, NameError> {
		Name::new(name)
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

// }}}

// NameBuf {{{

/// An owned extended attribute name.
#[cfg(any(doc, feature = "alloc"))]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NameBuf {
	bytes: vec::Vec<u8>,
}

#[cfg(any(doc, feature = "alloc"))]
impl NameBuf {
	/// Attempts to allocate a new `NameBuf` containing the given extended
	/// attribute name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// [`Name::MAX_LEN`] bytes, or contains `NUL`.
	pub fn new(name: &str) -> Result<NameBuf, NameError> {
		Name::new(name).map(Name::to_name_buf)
	}

	/// Borrows this `NameBuf` as a [`Name`].
	#[must_use]
	pub fn as_name(&self) -> &Name {
		unsafe { Name::from_bytes_unchecked(&self.bytes) }
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl AsRef<Name> for NameBuf {
	fn as_ref(&self) -> &Name {
		self.as_name()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl borrow::Borrow<Name> for NameBuf {
	fn borrow(&self) -> &Name {
		self.as_name()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl core::ops::Deref for NameBuf {
	type Target = Name;
	fn deref(&self) -> &Name {
		self.as_name()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<&Name> for NameBuf {
	fn from(name: &Name) -> Self {
		name.to_name_buf()
	}
}

#[cfg(any(doc, feature = "std"))]
impl From<&Name> for sync::Arc<NameBuf> {
	fn from(name: &Name) -> Self {
		sync::Arc::new(name.into())
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<&Name> for boxed::Box<NameBuf> {
	fn from(name: &Name) -> Self {
		boxed::Box::new(name.into())
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<&Name> for rc::Rc<NameBuf> {
	fn from(name: &Name) -> Self {
		rc::Rc::new(name.into())
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<borrow::Cow<'_, Name>> for NameBuf {
	fn from(name: borrow::Cow<Name>) -> Self {
		match name {
			borrow::Cow::Owned(name) => name,
			borrow::Cow::Borrowed(name) => name.to_name_buf(),
		}
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<boxed::Box<Name>> for NameBuf {
	fn from(name: boxed::Box<Name>) -> Self {
		name.into_name_buf()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl convert::TryFrom<string::String> for NameBuf {
	type Error = NameError;
	fn try_from(name: string::String) -> Result<NameBuf, NameError> {
		let bytes = name.into_bytes();
		Name::from_bytes(&bytes)?;
		Ok(NameBuf { bytes })
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl convert::TryFrom<NameBuf> for string::String {
	type Error = string::FromUtf8Error;
	fn try_from(
		name: NameBuf,
	) -> Result<string::String, string::FromUtf8Error> {
		string::String::from_utf8(name.bytes)
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<Name> for NameBuf {
	fn eq(&self, other: &Name) -> bool {
		self.as_name() == other
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<NameBuf> for Name {
	fn eq(&self, other: &NameBuf) -> bool {
		self == other.as_name()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<str> for NameBuf {
	fn eq(&self, other: &str) -> bool {
		self.as_name().as_bytes() == other.as_bytes()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<NameBuf> for str {
	fn eq(&self, other: &NameBuf) -> bool {
		self.as_bytes() == other.as_name().as_bytes()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<[u8]> for NameBuf {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_name().as_bytes() == other
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<NameBuf> for [u8] {
	fn eq(&self, other: &NameBuf) -> bool {
		self == other.as_name().as_bytes()
	}
}

// }}}

// ValueError {{{

/// Errors that may occur when validating an extended attribute value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ValueError {
	/// The input length in bytes exceeds [`Value::MAX_LEN`].
	ExceedsMaxLen,
}

// }}}

// Value {{{

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

	/// Converts a `Value` to an owned [`ValueBuf`].
	#[cfg(any(doc, feature = "alloc"))]
	#[must_use]
	pub fn to_value_buf(&self) -> ValueBuf {
		ValueBuf {
			bytes: self.bytes.to_vec(),
		}
	}

	/// Converts a [`Box<Value>`] into a [`ValueBuf`] without copying or
	/// allocating.
	///
	/// [`Box<Value>`]: boxed::Box<Value>
	#[cfg(any(doc, feature = "alloc"))]
	#[must_use]
	pub fn into_value_buf(self: boxed::Box<Value>) -> ValueBuf {
		let raw = boxed::Box::into_raw(self) as *mut [u8];
		ValueBuf {
			bytes: vec::Vec::from(unsafe { boxed::Box::from_raw(raw) }),
		}
	}
}

impl fmt::Debug for Value {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bytes.fmt(fmt)
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl borrow::ToOwned for Value {
	type Owned = ValueBuf;
	fn to_owned(&self) -> ValueBuf {
		self.to_value_buf()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl<'a> From<&'a Value> for borrow::Cow<'a, Value> {
	fn from(value: &'a Value) -> Self {
		borrow::Cow::Borrowed(value)
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

// }}}

// ValueBuf {{{

/// An owned extended attribute value.
#[cfg(any(doc, feature = "alloc"))]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValueBuf {
	bytes: vec::Vec<u8>,
}

#[cfg(any(doc, feature = "alloc"))]
impl ValueBuf {
	/// Attempts to allocate a new `ValueBuf` containing the given extended
	/// attribute value.
	///
	/// # Errors
	///
	/// Returns an error if the slice is longer than [`Value::MAX_LEN`] bytes.
	pub fn new(value: &[u8]) -> Result<ValueBuf, ValueError> {
		Value::new(value).map(Value::to_value_buf)
	}

	/// Borrows this `ValueBuf` as a [`Value`].
	#[must_use]
	pub fn as_value(&self) -> &Value {
		unsafe { Value::new_unchecked(&self.bytes) }
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl AsRef<Value> for ValueBuf {
	fn as_ref(&self) -> &Value {
		self.as_value()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl borrow::Borrow<Value> for ValueBuf {
	fn borrow(&self) -> &Value {
		self.as_value()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl core::ops::Deref for ValueBuf {
	type Target = Value;
	fn deref(&self) -> &Value {
		self.as_value()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<&Value> for ValueBuf {
	fn from(value: &Value) -> Self {
		value.to_value_buf()
	}
}

#[cfg(any(doc, feature = "std"))]
impl From<&Value> for sync::Arc<ValueBuf> {
	fn from(value: &Value) -> Self {
		sync::Arc::new(value.into())
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<&Value> for boxed::Box<ValueBuf> {
	fn from(value: &Value) -> Self {
		boxed::Box::new(value.into())
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<&Value> for rc::Rc<ValueBuf> {
	fn from(value: &Value) -> Self {
		rc::Rc::new(value.into())
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<borrow::Cow<'_, Value>> for ValueBuf {
	fn from(value: borrow::Cow<Value>) -> Self {
		match value {
			borrow::Cow::Owned(value) => value,
			borrow::Cow::Borrowed(value) => value.to_value_buf(),
		}
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl From<boxed::Box<Value>> for ValueBuf {
	fn from(value: boxed::Box<Value>) -> Self {
		value.into_value_buf()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<Value> for ValueBuf {
	fn eq(&self, other: &Value) -> bool {
		self.as_value() == other
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<ValueBuf> for Value {
	fn eq(&self, other: &ValueBuf) -> bool {
		self == other.as_value()
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<[u8]> for ValueBuf {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_value().as_bytes() == other
	}
}

#[cfg(any(doc, feature = "alloc"))]
impl PartialEq<ValueBuf> for [u8] {
	fn eq(&self, other: &ValueBuf) -> bool {
		self == other.as_value().as_bytes()
	}
}

// }}}
