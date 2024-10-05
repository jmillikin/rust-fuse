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

use core::fmt;
use core::ops;

const S_IFMT: u32 = 0xF000;

/// Representation of Unix file modes.
///
/// Unix file modes are a bitmask comprising a [`FileType`], permission bits,
/// and additional flags such as the [sticky bit].
///
/// The lowest 9 mode bits (the "permission bits") are standardized by POSIX.
/// Presence and interpretation of other bits is platform-specific.
///
/// [sticky bit]: https://en.wikipedia.org/wiki/Sticky_bit
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileMode {
	bits: u32,
}

impl FileMode {
	/// FileMode mask for a [`Directory`](FileType::Directory) node.
	pub const S_IFDIR: FileMode = FileType::Directory.as_mode();

	/// FileMode mask for a [`CharacterDevice`](FileType::CharacterDevice) node.
	pub const S_IFCHR: FileMode = FileType::CharacterDevice.as_mode();

	/// FileMode mask for a [`BlockDevice`](FileType::BlockDevice) node.
	pub const S_IFBLK: FileMode = FileType::BlockDevice.as_mode();

	/// FileMode mask for a [`Regular`](FileType::Regular) node.
	pub const S_IFREG: FileMode = FileType::Regular.as_mode();

	/// FileMode mask for a [`Symlink`](FileType::Symlink) node.
	pub const S_IFLNK: FileMode = FileType::Symlink.as_mode();

	/// FileMode mask for a [`Socket`](FileType::Socket) node.
	pub const S_IFSOCK: FileMode = FileType::Socket.as_mode();

	/// FileMode mask for a [`NamedPipe`](FileType::NamedPipe) node.
	pub const S_IFIFO: FileMode = FileType::NamedPipe.as_mode();
}

impl FileMode {
	/// Creates a new `FileMode` with the given value.
	#[inline]
	#[must_use]
	pub const fn new(mode: u32) -> FileMode {
		Self { bits: mode }
	}

	/// Returns the mode as a primitive integer.
	#[inline]
	#[must_use]
	pub const fn get(self) -> u32 {
		self.bits
	}

	/// Returns the permission bits set in the mode.
	///
	/// The permission bits are the lowest 9 bits (mask `0o777`).
	#[inline]
	#[must_use]
	pub const fn permissions(self) -> u32 {
		self.bits & 0o777
	}

	#[inline]
	#[must_use]
	pub(crate) const fn type_bits(self) -> u32 {
		self.bits & S_IFMT
	}
}

impl fmt::Debug for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{:#o}", self.bits)
	}
}

impl fmt::Binary for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl ops::BitOr<u32> for FileMode {
	type Output = FileMode;

	fn bitor(self, rhs: u32) -> FileMode {
		FileMode {
			bits: self.bits | rhs,
		}
	}
}

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
pub enum FileType {
	/// A [block device] (`DT_BLK`).
	///
	/// [block device]: https://en.wikipedia.org/wiki/Device_file#Block_devices
	BlockDevice,

	/// A [character device] (`DT_CHR`).
	///
	/// [character device]: https://en.wikipedia.org/wiki/Device_file#Character_devices
	CharacterDevice,

	/// A directory (`DT_DIR`)
	Directory,

	/// A [named pipe] (`DT_FIFO`).
	///
	/// [named pipe]: https://en.wikipedia.org/wiki/Named_pipe
	NamedPipe,

	/// A regular file (`DT_REG`).
	Regular,

	/// A [Unix domain socket] (`DT_SOCK`).
	///
	/// [Unix domain socket]: https://en.wikipedia.org/wiki/Unix_domain_socket
	Socket,

	/// A [symbolic link] (`DT_LNK`).
	///
	/// [symbolic link]: https://en.wikipedia.org/wiki/Symbolic_link
	Symlink,
}

impl FileType {
	const DT_FIFO: u32 =  1;
	const DT_CHR:  u32 =  2;
	const DT_DIR:  u32 =  4;
	const DT_BLK:  u32 =  6;
	const DT_REG:  u32 =  8;
	const DT_LNK:  u32 = 10;
	const DT_SOCK: u32 = 12;

	/// Returns the `FileType` contained in a Unix file mode.
	///
	/// Returns `None` if the mode's type bits are an unknown value. To support
	/// platform-specific file types, the user should inspect the mode directly.
	#[inline]
	#[must_use]
	pub const fn from_mode(mode: FileMode) -> Option<FileType> {
		FileType::from_bits(mode.type_bits() >> 12)
	}

	/// Returns the type encoded as a Unix file mode.
	#[inline]
	#[must_use]
	pub const fn as_mode(self) -> FileMode {
		FileMode::new(self.as_bits() << 12)
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
	pub(crate) const fn from_bits(bits: u32) -> Option<FileType> {
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

impl From<FileType> for FileMode {
	fn from(file_type: FileType) -> FileMode {
		file_type.as_mode()
	}
}
