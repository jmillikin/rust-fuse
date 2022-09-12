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

use core::fmt;

use crate::protocol::common::FileMode;

#[repr(u32)]
#[non_exhaustive]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FileType {
	/// `DT_UNKNOWN`
	Unknown,

	/// `DT_FIFO`
	NamedPipe,

	/// `DT_CHR`
	CharDevice,

	/// `DT_DIR`
	Directory,

	/// `DT_BLK`
	BlockDevice,

	/// `DT_REG`
	Regular,

	/// `DT_LNK`
	Symlink,

	/// `DT_SOCK`
	Socket,

	/// `DT_WHT`
	Whiteout,
}

const DT_UNKNOWN : u32 =  0;
const DT_FIFO    : u32 =  1;
const DT_CHR     : u32 =  2;
const DT_DIR     : u32 =  4;
const DT_BLK     : u32 =  6;
const DT_REG     : u32 =  8;
const DT_LNK     : u32 = 10;
const DT_SOCK    : u32 = 12;
const DT_WHT     : u32 = 14;

impl FileType {
	pub(crate) fn from_mode(mode: FileMode) -> Option<FileType> {
		Self::from_bits((mode.0 >> 12) & 0xF)
	}

	pub(crate) fn from_bits(bits: u32) -> Option<FileType> {
		match bits {
			DT_UNKNOWN => Some(Self::Unknown),
			DT_FIFO    => Some(Self::NamedPipe),
			DT_CHR     => Some(Self::CharDevice),
			DT_DIR     => Some(Self::Directory),
			DT_BLK     => Some(Self::BlockDevice),
			DT_REG     => Some(Self::Regular),
			DT_LNK     => Some(Self::Symlink),
			DT_SOCK    => Some(Self::Socket),
			DT_WHT     => Some(Self::Whiteout),
			_          => None,
		}
	}

	pub(crate) fn as_bits(&self) -> u32 {
		match *self {
			FileType::Unknown     => DT_UNKNOWN,
			FileType::NamedPipe   => DT_FIFO,
			FileType::CharDevice  => DT_CHR,
			FileType::Directory   => DT_DIR,
			FileType::BlockDevice => DT_BLK,
			FileType::Regular     => DT_REG,
			FileType::Symlink     => DT_LNK,
			FileType::Socket      => DT_SOCK,
			FileType::Whiteout    => DT_WHT,
		}
	}

	pub(crate) fn mode_bits(&self) -> u32 {
		self.as_bits() << 12
	}
}

impl fmt::Debug for FileType {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for FileType {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			FileType::Unknown     => fmt.write_str("Unknown"),
			FileType::NamedPipe   => fmt.write_str("NamedPipe"),
			FileType::CharDevice  => fmt.write_str("CharDevice"),
			FileType::Directory   => fmt.write_str("Directory"),
			FileType::BlockDevice => fmt.write_str("BlockDevice"),
			FileType::Regular     => fmt.write_str("Regular"),
			FileType::Symlink     => fmt.write_str("Symlink"),
			FileType::Socket      => fmt.write_str("Socket"),
			FileType::Whiteout    => fmt.write_str("Whiteout"),
		}
	}
}
