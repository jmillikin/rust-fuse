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

//! Character devices in userspace (CUSE).
//!
//! A CUSE device is a character device that proxies operations through the
//! kernel into a userspace server process. The protocol is approximately a
//! subset of FUSE, except for the CUSE-only "unrestricted ioctl" feature.
//!
//! At present only Linux supports creating CUSE devices. The FreeBSD kernel
//! contains a driver for userspace character devices, which they also call
//! CUSE, but it implements a different API that is unrelated to FUSE.

use core::fmt;

use crate::internal::debug;

// Matches the `limits.h` constant for Linux.
//
// Note that the kernel doesn't enforce a limit on character device names
// other than needing to fit within the CUSE_INIT response. Setting a lower
// limit in this library avoids creating sysfs entries that are too long for
// some programs name, and also leaves a path open to formatting CUSE_INIT
// response data into a stack-allocated `[u8; PAGE_SIZE]` buffer in the future.
const NAME_MAX: usize = 255;

// DeviceNameError {{{

/// Errors that may occur when validating a CUSE device name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DeviceNameError {
	/// The input is empty.
	Empty,
	/// The input contains `NUL`.
	ContainsNul,
	/// The input contains `'/'`.
	ContainsSlash,
	/// The input length in bytes exceeds `NAME_MAX`.
	ExceedsMaxLen,
}

// }}}

// DeviceName {{{

/// A borrowed CUSE device name.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the name of a CUSE device. It can be constructed safely from a `&str` or
/// `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is non-empty, is less than `NAME_MAX` bytes in length, and does not
/// contain a forbidden character (`NUL` or `'/'`).
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceName {
	bytes: [u8],
}

impl DeviceName {
	/// Attempts to reborrow a string as a CUSE character device name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// `NAME_MAX` bytes, or contains a forbidden character (`NUL` or `'/'`).
	#[inline]
	pub fn new(name: &str) -> Result<&DeviceName, DeviceNameError> {
		Self::from_bytes(name.as_bytes())
	}

	/// Reborrows a string as a CUSE character device name, without validation.
	///
	/// # Safety
	///
	/// The provided string must be non-empty, must be no longer than
	/// `NAME_MAX` bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(name: &str) -> &DeviceName {
		Self::from_bytes_unchecked(name.as_bytes())
	}

	/// Attempts to reborrow a byte slice as a CUSE character device name.
	///
	/// # Errors
	///
	/// Returns an error if the slice is empty, is longer than
	/// `NAME_MAX` bytes, or contains a forbidden byte (`0x00` or `0x2F`).
	#[inline]
	pub fn from_bytes(bytes: &[u8]) -> Result<&DeviceName, DeviceNameError> {
		if bytes.is_empty() {
			return Err(DeviceNameError::Empty);
		}
		if bytes.len() > NAME_MAX {
			return Err(DeviceNameError::ExceedsMaxLen);
		}
		for &byte in bytes {
			if byte == 0 {
				return Err(DeviceNameError::ContainsNul);
			}
			if byte == b'/' {
				return Err(DeviceNameError::ContainsSlash);
			}
		}
		Ok(unsafe { Self::from_bytes_unchecked(bytes) })
	}

	/// Reborrows a byte slice as a CUSE character device name, without
	/// validation.
	///
	/// # Safety
	///
	/// The provided `&[u8]` must be non-empty, must be no longer than
	/// `NAME_MAX` bytes, and must not contain a forbidden byte
	/// (`0x00` or `0x2F`).
	#[inline]
	#[must_use]
	pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &DeviceName {
		&*(bytes as *const [u8] as *const DeviceName)
	}

	/// Converts this `DeviceName` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Attempts to convert this `DeviceName` to a `&str`.
	///
	/// # Errors
	///
	/// Returns an error if the device name is not UTF-8.
	#[inline]
	pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.bytes)
	}
}

impl fmt::Debug for DeviceName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::bytes(&self.bytes).fmt(fmt)
	}
}

impl PartialEq<str> for DeviceName {
	fn eq(&self, other: &str) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for DeviceName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<DeviceName> for str {
	fn eq(&self, other: &DeviceName) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<DeviceName> for [u8] {
	fn eq(&self, other: &DeviceName) -> bool {
		self.eq(other.as_bytes())
	}
}

// }}}

// DeviceNumber {{{

/// A Unix device number.
///
/// Device numbers are a tuple of the "major" and "minor" numbers. The exact
/// semantics of these values are platform-specific, but in general the major
/// number identifies a category of device driver and the minor number
/// identifies a specific device.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceNumber {
	major: u32,
	minor: u32,
}

impl DeviceNumber {
	/// Create a new `DeviceNumber` with the given major and minor numbers.
	#[inline]
	#[must_use]
	pub const fn new(major: u32, minor: u32) -> DeviceNumber {
		Self { major, minor }
	}

	/// Return the device number's major number.
	#[inline]
	#[must_use]
	pub const fn major(&self) -> u32 {
		self.major
	}

	/// Return the device number's minor number.
	#[inline]
	#[must_use]
	pub const fn minor(&self) -> u32 {
		self.minor
	}
}

impl PartialEq<(u32, u32)> for DeviceNumber {
	fn eq(&self, other: &(u32, u32)) -> bool {
		(self.major, self.minor) == *other
	}
}

impl PartialEq<DeviceNumber> for (u32, u32) {
	fn eq(&self, other: &DeviceNumber) -> bool {
		(other.major, other.minor) == *self
	}
}

// }}}
