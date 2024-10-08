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

/// The maximum length of a node name, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on node names.
///
/// | Platform | Symbolic constant | Value |
/// |----------|-------------------|-------|
/// | FreeBSD  | `NAME_MAX`        | 255   |
/// | Linux    | `FUSE_NAME_MAX`   | 1024  |
pub const NAME_MAX: Option<usize> = name_max();

const fn name_max() -> Option<usize> {
	if cfg!(target_os = "freebsd") {
		Some(255) // NAME_MAX
	} else if cfg!(target_os = "linux") {
		Some(1024) // FUSE_NAME_MAX
	} else {
		None
	}
}

/// Errors that may occur when validating the content of a node name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NodeNameError {
	/// The input is empty.
	Empty,
	/// The input contains `NUL`.
	ContainsNul,
	/// The input contains `'/'`.
	ContainsSlash,
	/// The input length in bytes exceeds [`NAME_MAX`].
	ExceedsMaxLen,
}

/// A borrowed filesystem node name.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the name of a filesystem node. It can be constructed safely from a `&str`
/// or `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is non-empty, is less than [`NAME_MAX`] bytes in length, and
/// does not contain a forbidden character (`NUL` or `'/'`).
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeName {
	bytes: [u8],
}

impl NodeName {
	/// Attempts to reborrow a string as a node name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// [`NAME_MAX`] bytes, or contains a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	pub fn new(name: &str) -> Result<&NodeName, NodeNameError> {
		Self::from_bytes(name.as_bytes())
	}

	/// Reborrows a string as a node name, without validation.
	///
	/// # Safety
	///
	/// The provided string must be non-empty, must be no longer than
	/// [`NAME_MAX`] bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub unsafe fn new_unchecked(name: &str) -> &NodeName {
		Self::from_bytes_unchecked(name.as_bytes())
	}

	/// Attempts to reborrow a byte slice as a node name.
	///
	/// # Errors
	///
	/// Returns an error if the slice is empty, is longer than
	/// [`NAME_MAX`] bytes, or contains a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	pub fn from_bytes(bytes: &[u8]) -> Result<&NodeName, NodeNameError> {
		if bytes.is_empty() {
			return Err(NodeNameError::Empty);
		}
		if let Some(max_len) = NAME_MAX {
			if bytes.len() > max_len {
				return Err(NodeNameError::ExceedsMaxLen);
			}
		}
		for &byte in bytes {
			if byte == 0 {
				return Err(NodeNameError::ContainsNul);
			}
			if byte == b'/' {
				return Err(NodeNameError::ContainsSlash);
			}
		}
		Ok(unsafe { Self::from_bytes_unchecked(bytes) })
	}

	/// Reborrows a byte slice as a node name, without validation.
	///
	/// # Safety
	///
	/// The provided slice must be non-empty, must be no longer than
	/// [`NAME_MAX`] bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &NodeName {
		&*(bytes as *const [u8] as *const NodeName)
	}

	/// Converts this `NodeName` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Attempts to convert this `NodeName` to a `&str`.
	///
	/// # Errors
	///
	/// Returns an error if the name is not UTF-8.
	#[inline]
	pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.bytes)
	}
}

impl fmt::Debug for NodeName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::bytes(&self.bytes).fmt(fmt)
	}
}

impl<'a> convert::TryFrom<&'a NodeName> for &'a str {
	type Error = core::str::Utf8Error;
	fn try_from(name: &'a NodeName) -> Result<&'a str, core::str::Utf8Error> {
		name.as_str()
	}
}

impl<'a> convert::TryFrom<&'a str> for &'a NodeName {
	type Error = NodeNameError;
	fn try_from(name: &'a str) -> Result<&'a NodeName, NodeNameError> {
		NodeName::new(name)
	}
}

impl PartialEq<str> for NodeName {
	fn eq(&self, other: &str) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for NodeName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<NodeName> for str {
	fn eq(&self, other: &NodeName) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<NodeName> for [u8] {
	fn eq(&self, other: &NodeName) -> bool {
		self.eq(other.as_bytes())
	}
}
