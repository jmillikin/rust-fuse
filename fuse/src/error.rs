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

use core::{fmt, num};

#[cfg(feature = "std")]
use crate::internal::fuse_kernel;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error {
	kind: ErrorKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
	MissingNodeId,
	UnexpectedEof,
	ExpectedCuseInit(u32),
	ExpectedFuseInit(u32),
	InvalidLockType,
	OpcodeMismatch,
}

impl Error {
	pub(crate) fn expected_cuse_init(opcode: u32) -> Error {
		Error {
			kind: ErrorKind::ExpectedCuseInit(opcode),
		}
	}

	pub(crate) fn expected_fuse_init(opcode: u32) -> Error {
		Error {
			kind: ErrorKind::ExpectedFuseInit(opcode),
		}
	}

	pub(crate) fn invalid_lock_type() -> Error {
		Error {
			kind: ErrorKind::InvalidLockType,
		}
	}

	pub(crate) fn missing_node_id() -> Error {
		Error {
			kind: ErrorKind::MissingNodeId,
		}
	}

	pub(crate) fn unexpected_eof() -> Error {
		Error {
			kind: ErrorKind::UnexpectedEof,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		fmt::Debug::fmt(self, fmt)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<crate::io::RequestError> for Error {
	fn from(err: crate::io::RequestError) -> Error {
		use crate::io::RequestError;
		match err {
			RequestError::InvalidLockType => Self::invalid_lock_type(),
			RequestError::MissingNodeId => Self::missing_node_id(),
			RequestError::OpcodeMismatch => Self {
				kind: ErrorKind::OpcodeMismatch,
			},
			RequestError::UnexpectedEof => Self::unexpected_eof(),
		}
	}
}

#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
impl From<Error> for std::io::Error {
	fn from(err: Error) -> std::io::Error {
		use std::io;
		match err.kind {
			ErrorKind::MissingNodeId => io::Error::new(
				io::ErrorKind::InvalidData,
				"Request field 'fuse_in_header::nodeid' is missing (expected non-zero)",
			),
			ErrorKind::UnexpectedEof => io::ErrorKind::UnexpectedEof.into(),
			ErrorKind::ExpectedCuseInit(opcode) => io::Error::new(
				io::ErrorKind::InvalidData,
				format!(
					"Received opcode {:?} from kernel (expected CUSE_INIT)",
					fuse_kernel::fuse_opcode(opcode),
				),
			),
			ErrorKind::ExpectedFuseInit(opcode) => io::Error::new(
				io::ErrorKind::InvalidData,
				format!(
					"Received opcode {:?} from kernel (expected FUSE_INIT)",
					fuse_kernel::fuse_opcode(opcode),
				),
			),
			ErrorKind::InvalidLockType => io::Error::new(
				io::ErrorKind::InvalidData,
				"Invalid fcntl() lock type (expected F_RDLCK or F_WRLCK)",
			),
			ErrorKind::OpcodeMismatch => io::Error::new(
				io::ErrorKind::InvalidData,
				"Opcode mismatch",
			),
		}
	}
}

#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorCode(num::NonZeroU16);

impl ErrorCode {
	pub fn new(n: num::NonZeroU16) -> ErrorCode {
		ErrorCode(n)
	}

	pub fn name(&self) -> Option<&'static str> {
		self.name_impl()
	}
}

impl From<num::NonZeroU16> for ErrorCode {
	fn from(n: num::NonZeroU16) -> ErrorCode {
		ErrorCode(n)
	}
}

impl From<ErrorCode> for u16 {
	fn from(err: ErrorCode) -> u16 {
		err.0.get()
	}
}

impl From<ErrorCode> for num::NonZeroU16 {
	fn from(err: ErrorCode) -> num::NonZeroU16 {
		err.0
	}
}

impl From<ErrorCode> for u32 {
	fn from(err: ErrorCode) -> u32 {
		err.0.get().into()
	}
}

impl From<ErrorCode> for num::NonZeroU32 {
	fn from(err: ErrorCode) -> num::NonZeroU32 {
		err.0.into()
	}
}

impl From<ErrorCode> for i32 {
	fn from(err: ErrorCode) -> i32 {
		err.0.get().into()
	}
}

impl From<ErrorCode> for num::NonZeroI32 {
	fn from(err: ErrorCode) -> num::NonZeroI32 {
		err.0.into()
	}
}

impl From<ErrorCode> for u64 {
	fn from(err: ErrorCode) -> u64 {
		err.0.get().into()
	}
}

impl From<ErrorCode> for num::NonZeroU64 {
	fn from(err: ErrorCode) -> num::NonZeroU64 {
		err.0.into()
	}
}

impl From<ErrorCode> for i64 {
	fn from(err: ErrorCode) -> i64 {
		err.0.get().into()
	}
}

impl From<ErrorCode> for num::NonZeroI64 {
	fn from(err: ErrorCode) -> num::NonZeroI64 {
		err.0.into()
	}
}

impl fmt::Debug for ErrorCode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		self.0.fmt(fmt)
	}
}

impl fmt::Display for ErrorCode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		self.0.fmt(fmt)
	}
}

impl fmt::Binary for ErrorCode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::LowerHex for ErrorCode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::UpperHex for ErrorCode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

macro_rules! impl_partial_eq {
	($t:ty) => {
		impl PartialEq<$t> for ErrorCode {
			fn eq(&self, x: &$t) -> bool {
				<$t>::from(self.0.get()) == *x
			}
		}

		impl PartialEq<ErrorCode> for $t {
			fn eq(&self, x: &ErrorCode) -> bool {
				<$t>::from(x.0.get()) == *self
			}
		}
	};
}

impl_partial_eq!(i32);
impl_partial_eq!(i64);
impl_partial_eq!(u16);
impl_partial_eq!(u32);
impl_partial_eq!(u64);
impl_partial_eq!(usize);

impl PartialEq<i16> for ErrorCode {
	fn eq(&self, x: &i16) -> bool {
		if *x <= 0 {
			return false;
		}
		self.0.get() == *x as u16
	}
}

impl PartialEq<ErrorCode> for i16 {
	fn eq(&self, x: &ErrorCode) -> bool {
		if *self <= 0 {
			return false;
		}
		x.0.get() == *self as u16
	}
}

impl PartialEq<isize> for ErrorCode {
	fn eq(&self, x: &isize) -> bool {
		if *x <= 0 {
			return false;
		}
		usize::from(self.0.get()) == *x as usize
	}
}

impl PartialEq<ErrorCode> for isize {
	fn eq(&self, x: &ErrorCode) -> bool {
		if *self <= 0 {
			return false;
		}
		usize::from(x.0.get()) == *self as usize
	}
}

impl ErrorCode {
	pub const EINTR: ErrorCode = target::EINTR;
	pub const EIO: ErrorCode = target::EIO;
	pub const E2BIG: ErrorCode = target::E2BIG;
	pub const ENODEV: ErrorCode = target::ENODEV;
	pub const ENOENT: ErrorCode = target::ENOENT;
	pub const ENOSYS: ErrorCode = target::ENOSYS;
	pub const ERANGE: ErrorCode = target::ERANGE;
	pub const ENOATTR: ErrorCode = target::ENOATTR;

	fn name_impl(&self) -> Option<&'static str> {
		match *self {
			Self::EINTR => Some("EINTR"),
			Self::EIO => Some("EIO"),
			Self::E2BIG => Some("E2BIG"),
			Self::ENODEV => Some("ENODEV"),
			Self::ENOENT => Some("ENOENT"),
			Self::ENOSYS => Some("ENOSYS"),
			Self::ERANGE => Some("ERANGE"),
			Self::ENOATTR => Some("ENOATTR"),
			_ => None,
		}
	}
}

macro_rules! target_error_codes {
	($( $name:ident : $value:literal , )*) => {
		mod target {
			$(
				pub(super) const $name: super::ErrorCode = super::ErrorCode(unsafe{
					core::num::NonZeroU16::new_unchecked($value)
				});
			)*
		}
	}
}

#[cfg(target_os = "freebsd")]
target_error_codes! {
	EINTR: 4,
	EIO: 5,
	E2BIG: 7,
	ENODEV: 19,
	ENOENT: 2,
	ENOSYS: 78,
	ERANGE: 34,
	ENOATTR: 87,
}

#[cfg(all(
	target_os = "linux",
	any(
		target_arch = "arm",
		target_arch = "x86",
		target_arch = "x86_64",
	),
))]
target_error_codes! {
	EINTR: 4,
	EIO: 5,
	E2BIG: 7,
	ENODEV: 19,
	ENOENT: 2,
	ENOSYS: 38,
	ERANGE: 34,
	ENOATTR: 61,
}
