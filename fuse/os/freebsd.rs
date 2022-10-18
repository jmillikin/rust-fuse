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

//! FreeBSD-specific functionality.

use core::ffi;
use core::fmt;

// FuseSubtype {{{

/// A borrowed FUSE filesystem subtype.
///
/// This value is passed to `nmount()` with the key `subtype=` when mounting
/// a FUSE filesystem.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FuseSubtype {
	inner: ffi::CStr,
}

impl FuseSubtype {
	/// Attempts to reborrow a C string as a FUSE filesystem subtype.
	///
	/// # Errors
	///
	/// Returns `None` if the C string is empty.
	#[must_use]
	pub fn new(subtype: &ffi::CStr) -> Option<&FuseSubtype> {
		if cstr_is_empty(subtype) {
			return None;
		}
		Some(unsafe { Self::new_unchecked(subtype) })
	}

	/// Reborrows a C string as a FUSE filesystem subtype, without validation.
	///
	/// # Safety
	///
	/// The provided C string must be non-empty.
	#[must_use]
	pub const unsafe fn new_unchecked(subtype: &ffi::CStr) -> &FuseSubtype {
		&*(subtype as *const ffi::CStr as *const FuseSubtype)
	}

	/// Returns this FUSE filesystem subtype as a borrowed C string.
	#[must_use]
	pub const fn as_cstr(&self) -> &ffi::CStr {
		&self.inner
	}
}

impl fmt::Debug for FuseSubtype {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.inner, fmt)
	}
}

// }}}

// MountOptions {{{

/// Builder for FreeBSD `nmount()` options.
#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	default_permissions: bool,
	subtype: Option<&'a FuseSubtype>,
}

impl<'a> MountOptions<'a> {
	/// Create a new `MountOptions` with default values.
	#[must_use]
	pub fn new() -> Self {
		MountOptions {
			default_permissions: false,
			subtype: None,
		}
	}

	/// Returns the `default_permissions` mount option.
	///
	/// If true, then the kernel will perform its own permission checking
	/// in addition to any permission checks by the filesystem.
	#[must_use]
	pub fn default_permissions(&self) -> bool {
		self.default_permissions
	}

	/// Sets the `default_permissions` mount option.
	pub fn set_default_permissions(&mut self, default_permissions: bool) {
		self.default_permissions = default_permissions;
	}

	/// Returns the `subtype=` mount option.
	#[must_use]
	pub fn subtype(&self) -> Option<&'a FuseSubtype> {
		self.subtype
	}

	/// Sets the `subtype=` mount option.
	pub fn set_subtype(&mut self, subtype: Option<&'a FuseSubtype>) {
		self.subtype = subtype;
	}
}

impl fmt::Debug for MountOptions<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MountOptions")
			.field("default_permissions", &self.default_permissions())
			.field("subtype", &format_args!("{:?}", self.subtype()))
			.finish()
	}
}

// }}}

// https://github.com/rust-lang/rust/issues/102444
fn cstr_is_empty(s: &ffi::CStr) -> bool {
	unsafe { s.as_ptr().read() == 0 }
}
