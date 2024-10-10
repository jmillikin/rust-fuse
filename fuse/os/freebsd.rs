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

#[cfg(target_os = "freebsd")]
use freebsd_errno as errno;

use crate::LockMode;

/// Shared (or 'read') locks may be held by any number of owners.
pub const F_RDLCK: LockMode = LockMode(1);

/// Exclusive (or 'write') locks may be held only by a single owner.
pub const F_WRLCK: LockMode = LockMode(3);

/// Absence or removal of a lock.
pub const F_UNLCK: LockMode = LockMode(2);

/// Adapter from FreeBSD error codes to FUSE errors.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OsError(pub errno::Error);

#[inline]
#[must_use]
const fn fuse_error(errno: errno::Error) -> crate::Error {
	use core::num::NonZeroI32;

	let errno = errno.get();
	let errno_neg = if errno < 0 || errno > u16::MAX as i32 {
		-(u16::MAX as i32)
	} else {
		errno.wrapping_neg()
	};
	crate::Error(unsafe { NonZeroI32::new_unchecked(errno_neg) })
}

impl From<errno::Error> for OsError {
	#[inline]
	fn from(errno: errno::Error) -> OsError {
		OsError(errno)
	}
}

impl From<OsError> for crate::Error {
	#[inline]
	fn from(errno: OsError) -> crate::Error {
		fuse_error(errno.0)
	}
}

#[cfg(all(doc, not(target_os = "freebsd")))]
mod errno {
	#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Error;
	impl Error {
		pub const fn get(self) -> i32 { 1 }
	}

	pub const E2BIG: Error = Error;
	pub const EINTR: Error = Error;
	pub const EINVAL: Error = Error;
	pub const ENOENT: Error = Error;
	pub const EOVERFLOW: Error = Error;
	pub const EPROTO: Error = Error;
	pub const EAGAIN: Error = Error;
	pub const ENOSYS: Error = Error;
}

impl OsError {
	pub(crate) const E2BIG: crate::Error = fuse_error(errno::E2BIG);

	/// An operation was interrupted.
	///
	/// This error can be returned from an operation to signal that it was
	/// interrupted by a `FUSE_INTERRUPT` request.
	///
	///  This error maps to `EINTR`.
	pub const INTERRUPTED: crate::Error = fuse_error(errno::EINTR);

	/// The client specified an invalid argument.
	///
	/// This error maps to `EINVAL`.
	pub const INVALID_ARGUMENT: crate::Error = fuse_error(errno::EINVAL);

	/// The requested file or directory does not exist.
	///
	/// This error maps to `ENOENT`.
	pub const NOT_FOUND: crate::Error = fuse_error(errno::ENOENT);

	/// A value is too large to store in its data type.
	///
	/// This error maps to `EOVERFLOW`.
	pub const OVERFLOW: crate::Error = fuse_error(errno::EOVERFLOW);

	/// A message that violates the FUSE protocol was sent or received.
	///
	/// This error maps to `EPROTO`.
	pub const PROTOCOL_ERROR: crate::Error = fuse_error(errno::EPROTO);

	/// The requested operation is temporarily unavailable.
	///
	/// This error maps to `EAGAIN`.
	pub const UNAVAILABLE: crate::Error = fuse_error(errno::EAGAIN);

	/// The requested operation is not implemented in this server.
	///
	/// This error maps to `ENOSYS`.
	pub const UNIMPLEMENTED: crate::Error = fuse_error(errno::ENOSYS);
}

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
		if subtype.is_empty() {
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
