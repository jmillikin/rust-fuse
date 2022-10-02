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

use core::num::{NonZeroI32, NonZeroU16};

#[cfg(target_os = "freebsd")]
use freebsd_errno as os_errno;

#[cfg(target_os = "linux")]
use linux_errno as os_errno;

/// The error type for FUSE operations.
///
/// The FUSE wire protocol represents errors as 32-bit signed integers, but
/// the client implementation in Linux rejects error numbers outside the
/// interval `[1, 512)`. Other implementations impose similar limits. This
/// library therefore represents FUSE errors as having the semantics of a
/// [`NonZeroU16`](core::num::NonZeroU16).
///
/// To provide an ergonomic API it must be possible for the user to pass
/// OS-specific error numbers to FUSE functions, but the size and signedness
/// of these error numbers varies between OSes. This means it's not appropriate
/// to use `Into<NonZeroU16>` trait bounds for error-related functions.
///
/// The `Error` type solves this by implementing `From` such that error numbers
/// not representable as `u16` are mapped to [`u16::MAX`]. The resulting
/// behavior will be *as if* the kernel had received (and rejected) the
/// original error number.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Error {
	fuse_error_code: NonZeroI32,
}

impl Error {
	/// Creates a new instance of [`Error`] from a [`NonZeroU16`].
	///
	/// No validation is performed on the error number. The user should
	/// ensure it will be accpted by the client.
	#[inline]
	#[must_use]
	pub const fn new(errno: NonZeroU16) -> Error {
		let raw_neg = (errno.get() as i32).saturating_neg();
		let fuse_error_code = unsafe { NonZeroI32::new_unchecked(raw_neg) };
		Error { fuse_error_code }
	}

	#[inline]
	#[must_use]
	pub(crate) const fn raw_fuse_error_code(self) -> i32 {
		self.fuse_error_code.get()
	}

	#[cfg(target_os = "linux")]
	#[inline]
	#[must_use]
	const fn from_errno(errno: os_errno::Error) -> Error {
		Error::new(errno.get_nonzero())
	}

	#[cfg(target_os = "freebsd")]
	#[inline]
	#[must_use]
	const fn from_errno(errno: os_errno::Error) -> Error {
		let raw: NonZeroI32 = errno.get_nonzero();
		let raw_i32 = raw.get();
		let raw_neg = if raw_i32 < 0 || raw_i32 > u16::MAX as i32 {
			-(u16::MAX as i32)
		} else {
			raw_i32.saturating_neg()
		};
		let fuse_error_code = unsafe { NonZeroI32::new_unchecked(raw_neg) };
		Error { fuse_error_code }
	}
}

impl From<os_errno::Error> for Error {
	/// Convert an OS-specific error number into a FUSE error number.
	#[inline]
	fn from(os_errno: os_errno::Error) -> Error {
		Error::from_errno(os_errno)
	}
}

// Use a wrapper module to stop rustdoc from rendering large useless constant
// definitions inline.
//
// The rustdoc project has gone back and forth about whether constants should
// have their definition shown inline in the docs. At present they seem to have
// reached a compromise by hiding them on module-level constants and showing
// them for associated constants.
//
// https://github.com/rust-lang/rust/pull/53409
// https://github.com/rust-lang/rust/pull/66221
mod errno {
	use super::{os_errno, Error};

	pub(super) const EINVAL: Error = Error::from_errno(os_errno::EINVAL);
	pub(super) const EIO: Error = Error::from_errno(os_errno::EIO);
	pub(super) const ENOENT: Error = Error::from_errno(os_errno::ENOENT);
	pub(super) const ENOSYS: Error = Error::from_errno(os_errno::ENOSYS);

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

impl Error {
	pub(crate) const EIO: Error = errno::EIO;

	/// The client specified an invalid argument.
	///
	/// This error maps to `EINVAL`.
	pub const INVALID_ARGUMENT: Error = errno::EINVAL;

	/// The requested file or directory does not exist.
	///
	/// This error maps to `ENOENT`.
	pub const NOT_FOUND: Error = errno::ENOENT;

	/// The requested operation is not implemented in this server.
	///
	/// This error maps to `ENOSYS`.
	pub const UNIMPLEMENTED: Error = errno::ENOSYS;

	/// The requested extended attribute does not exist.
	///
	/// This error maps to either `ENODATA` or `ENOATTR`, depending on the
	/// target platform.
	pub const XATTR_NOT_FOUND: Error = enodata_or_enoattr!();
}
