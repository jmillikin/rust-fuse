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

//! Filesystem nodes and node properties.

use core::fmt;
use core::num;

use crate::internal::fuse_kernel;

use crate::protocol::common::DebugBytesAsString;

// Id {{{

/// Node IDs are per-mount unique identifiers for filesystem nodes.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
	bits: num::NonZeroU64,
}

const FUSE_ROOT_ID: Id = unsafe {
	Id::new_unchecked(fuse_kernel::FUSE_ROOT_ID)
};

impl Id {
	/// The node ID of the root directory.
	pub const ROOT: Id = FUSE_ROOT_ID;

	/// Creates a new `Id` if the given value is not zero.
	#[inline]
	#[must_use]
	pub const fn new(id: u64) -> Option<Id> {
		match num::NonZeroU64::new(id) {
			Some(id) => Some(Id { bits: id }),
			None => None,
		}
	}

	/// Creates a new `Id` without checking that the given value is non-zero.
	///
	/// # Safety
	///
	/// The value must not be zero.
	///
	/// The `Id` struct is a wrapper around [`NonZeroU64`](num::NonZeroU64),
	/// so passing zero to this function is undefined behavior.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(id: u64) -> Id {
		Self {
			bits: num::NonZeroU64::new_unchecked(id),
		}
	}

	/// Returns the node ID as a primitive integer.
	#[inline]
	#[must_use]
	pub const fn get(&self) -> u64 {
		self.bits.get()
	}

	/// Returns whether the node ID is [`Id::ROOT`].
	#[inline]
	#[must_use]
	pub const fn is_root(&self) -> bool {
		self.bits.get() == fuse_kernel::FUSE_ROOT_ID
	}
}

impl fmt::Debug for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::Binary for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

// }}}

// Name {{{

/// Errors that may occur when validating a node name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NameError {
	/// The input is empty.
	Empty,
	/// The input contains `NUL`.
	ContainsNul,
	/// The input contains `'/'`.
	ContainsSlash,
	/// The input length in bytes exceeds [`Name::MAX_LEN`].
	ExceedsMaxLen,
}

/// A borrowed filesystem node name.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the name of a filesystem node. It can be constructed safely from a `&str`
/// or `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is non-empty, is less than [`Name::MAX_LEN`] bytes in length, and
/// does not contain a forbidden character (`NUL` or `'/'`).
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Name {
	bytes: [u8]
}

#[cfg(target_os = "freebsd")]
const NAME_MAX: usize = 255;

#[cfg(target_os = "freebsd")]
macro_rules! node_name_max_len {
	() => { Some(NAME_MAX) }
}

#[cfg(target_os = "linux")]
const FUSE_NAME_MAX: usize = 1024;

#[cfg(target_os = "linux")]
macro_rules! node_name_max_len {
	() => { Some(FUSE_NAME_MAX) }
}

impl Name {
	/// The maximum length of a node name, in bytes.
	///
	/// This value is platform-specific. If `None`, then the platform does not
	/// impose a maximum length on node names.
	///
	/// | Platform | Symbolic constant | Value |
	/// |----------|-------------------|-------|
	/// | FreeBSD  | `NAME_MAX`        | 255   |
	/// | Linux    | `FUSE_NAME_MAX`   | 1024  |
	///
	pub const MAX_LEN: Option<usize> = node_name_max_len!();

	/// Attempts to reborrow a string as a node name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// [`Name::MAX_LEN`] bytes, or contains a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	pub fn new(name: &str) -> Result<&Name, NameError> {
		Self::from_bytes(name.as_bytes())
	}

	/// Reborrows a string as a node name, without validation.
	///
	/// # Safety
	///
	/// The provided string must be non-empty, must be no longer than
	/// [`Name::MAX_LEN`] bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub unsafe fn new_unchecked(name: &str) -> &Name {
		Self::from_bytes_unchecked(name.as_bytes())
	}

	/// Attempts to reborrow a byte slice as a node name.
	///
	/// # Errors
	///
	/// Returns an error if the slice is empty, is longer than
	/// [`Name::MAX_LEN`] bytes, or contains a forbidden character
	/// (`NUL` or `'/'`).
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
		for &byte in bytes {
			if byte == 0 {
				return Err(NameError::ContainsNul);
			}
			if byte == b'/' {
				return Err(NameError::ContainsSlash);
			}
		}
		Ok(unsafe { Self::from_bytes_unchecked(bytes) })
	}

	/// Reborrows a byte slice as a node name, without validation.
	///
	/// # Safety
	///
	/// The provided slice must be non-empty, must be no longer than
	/// [`Name::MAX_LEN`] bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Name {
		&*(bytes as *const [u8] as *const Name)
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

impl fmt::Debug for Name {
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

// }}}

// Mode {{{

/// Representation of Unix file modes.
///
/// Unix file modes are a bitmask comprising a [`Type`], permission bits,
/// and additional flags such as the [sticky bit].
///
/// The lowest 9 mode bits (the "permission bits") are standardized by POSIX.
/// Presence and interpretation of other bits is platform-specific.
///
/// [sticky bit]: https://en.wikipedia.org/wiki/Sticky_bit
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mode {
	bits: u32,
}

impl Mode {
	/// Creates a new `Mode` with the given value.
	#[inline]
	#[must_use]
	pub const fn new(mode: u32) -> Mode {
		Self { bits: mode }
	}

	/// Returns the mode as a primitive integer.
	#[inline]
	#[must_use]
	pub const fn get(&self) -> u32 {
		self.bits
	}

	/// Returns the permission bits set in the mode.
	///
	/// The permission bits are the lowest 9 bits (mask `0o777`).
	#[inline]
	#[must_use]
	pub const fn permissions(&self) -> u32 {
		self.bits & 0o777
	}
}

impl fmt::Debug for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{:#o}", self.bits)
	}
}

impl fmt::Binary for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}


// }}}

// Type {{{

/// Representation of Unix file types.
///
/// A Unix file type identifies the purpose and capabilities of a filesystem
/// node. The POSIX standard describes seven file types; platforms may add
/// additional types as extensions.
///
/// Most FUSE filesystems will only create nodes of type `Directory`, `Regular`,
/// or `Symlink`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Type {
	/// A [block device].
	///
	/// [block device]: https://en.wikipedia.org/wiki/Device_file#Block_devices
	BlockDevice,

	/// A [character device].
	///
	/// [character device]: https://en.wikipedia.org/wiki/Device_file#Character_devices
	CharacterDevice,

	/// A directory.
	Directory,

	/// A [named pipe].
	///
	/// [named pipe]: https://en.wikipedia.org/wiki/Named_pipe
	NamedPipe,

	/// A regular file.
	Regular,

	/// A [Unix domain socket].
	///
	/// [Unix domain socket]: https://en.wikipedia.org/wiki/Unix_domain_socket
	Socket,

	/// A [symbolic link].
	///
	/// [symbolic link]: https://en.wikipedia.org/wiki/Symbolic_link
	Symlink,
}

const S_IFMT: u32 = 0xF000;

impl Type {
	const DT_FIFO: u32 =  1;
	const DT_CHR:  u32 =  2;
	const DT_DIR:  u32 =  4;
	const DT_BLK:  u32 =  6;
	const DT_REG:  u32 =  8;
	const DT_LNK:  u32 = 10;
	const DT_SOCK: u32 = 12;

	/// Returns the `Type` contained in a Unix file mode.
	///
	/// Returns `None` if the mode's type bits are an unknown value. To support
	/// platform-specific file types, the user should inspect the mode directly.
	#[inline]
	#[must_use]
	pub const fn from_mode(mode: Mode) -> Option<Type> {
		Type::from_bits((mode.bits & S_IFMT) >> 12)
	}

	#[inline]
	#[must_use]
	pub(crate) const fn as_bits(self) -> u32 {
		match self {
			Self::NamedPipe       => Self::DT_FIFO,
			Self::CharacterDevice => Self::DT_CHR,
			Self::Directory       => Self::DT_DIR,
			Self::BlockDevice     => Self::DT_BLK,
			Self::Regular         => Self::DT_REG,
			Self::Symlink         => Self::DT_LNK,
			Self::Socket          => Self::DT_SOCK,
		}
	}

	#[inline]
	#[must_use]
	pub(crate) const fn from_bits(bits: u32) -> Option<Type> {
		match bits {
			Self::DT_FIFO => Some(Self::NamedPipe),
			Self::DT_CHR  => Some(Self::CharacterDevice),
			Self::DT_DIR  => Some(Self::Directory),
			Self::DT_BLK  => Some(Self::BlockDevice),
			Self::DT_REG  => Some(Self::Regular),
			Self::DT_LNK  => Some(Self::Symlink),
			Self::DT_SOCK => Some(Self::Socket),
			_ => None,
		}
	}
}

// }}}
