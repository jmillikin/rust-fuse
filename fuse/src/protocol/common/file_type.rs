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

#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileType(pub(crate) u32);

const DT_UNKNOWN: u32 = 0;
const DT_FIFO: u32 = 1;
const DT_CHR: u32 = 2;
const DT_DIR: u32 = 4;
const DT_BLK: u32 = 6;
const DT_REG: u32 = 8;
const DT_LNK: u32 = 10;
const DT_SOCK: u32 = 12;
const DT_WHT: u32 = 14;

impl FileType {
	pub const UNKNOWN: FileType = FileType(DT_UNKNOWN);
	pub const FIFO: FileType = FileType(DT_FIFO);
	pub const CHR: FileType = FileType(DT_CHR);
	pub const DIR: FileType = FileType(DT_DIR);
	pub const BLK: FileType = FileType(DT_BLK);
	pub const REG: FileType = FileType(DT_REG);
	pub const LNK: FileType = FileType(DT_LNK);
	pub const SOCK: FileType = FileType(DT_SOCK);
	pub const WHT: FileType = FileType(DT_WHT);

	pub fn from_mode(mode: u32) -> Option<FileType> {
		match (mode >> 12) & 0xF {
			DT_UNKNOWN => Some(Self::UNKNOWN),
			DT_FIFO => Some(Self::FIFO),
			DT_CHR => Some(Self::CHR),
			DT_DIR => Some(Self::DIR),
			DT_BLK => Some(Self::BLK),
			DT_REG => Some(Self::REG),
			DT_LNK => Some(Self::LNK),
			DT_SOCK => Some(Self::SOCK),
			DT_WHT => Some(Self::WHT),
			_ => None,
		}
	}
}

impl core::ops::BitAnd<u32> for FileType {
	type Output = Option<FileType>;

	fn bitand(self, rhs: u32) -> Option<FileType> {
		if rhs & (self.0 << 12) != 0 {
			return Some(self);
		}
		None
	}
}

impl core::ops::BitAnd<FileType> for u32 {
	type Output = Option<FileType>;

	fn bitand(self, rhs: FileType) -> Option<FileType> {
		if self & (rhs.0 << 12) != 0 {
			return Some(rhs);
		}
		None
	}
}

impl core::ops::BitOr<u32> for FileType {
	type Output = u32;

	fn bitor(self, rhs: u32) -> u32 {
		(self.0 << 12) | rhs
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
			FileType::UNKNOWN => fmt.write_str("UNKNOWN"),
			FileType::FIFO => fmt.write_str("FIFO"),
			FileType::CHR => fmt.write_str("CHR"),
			FileType::DIR => fmt.write_str("DIR"),
			FileType::BLK => fmt.write_str("BLK"),
			FileType::REG => fmt.write_str("REG"),
			FileType::LNK => fmt.write_str("LNK"),
			FileType::SOCK => fmt.write_str("SOCK"),
			FileType::WHT => fmt.write_str("WHT"),
			_ => write!(fmt, "{:#010X}", self.0),
		}
	}
}

impl fmt::Binary for FileType {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::LowerHex for FileType {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::UpperHex for FileType {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}
