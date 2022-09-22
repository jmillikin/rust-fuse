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

use crate::protocol::common::FileType;

#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileMode(pub u32);

impl FileMode {
	pub fn file_type(&self) -> Option<FileType> {
		FileType::from_mode(*self)
	}
}

impl From<u32> for FileMode {
	fn from(mode: u32) -> Self {
		Self(mode)
	}
}

impl From<FileMode> for u32 {
	fn from(mode: FileMode) -> Self {
		mode.0
	}
}

impl core::ops::BitOr<FileType> for FileMode {
	type Output = FileMode;

	fn bitor(self, rhs: FileType) -> FileMode {
		FileMode(rhs.mode_bits() | self.0)
	}
}

impl core::ops::BitOr<FileMode> for FileType {
	type Output = FileMode;

	fn bitor(self, rhs: FileMode) -> FileMode {
		FileMode(self.mode_bits() | rhs.0)
	}
}

impl core::ops::BitOr<FileType> for u32 {
	type Output = FileMode;

	fn bitor(self, rhs: FileType) -> FileMode {
		FileMode(rhs.mode_bits() | self)
	}
}

impl core::ops::BitOr<u32> for FileType {
	type Output = FileMode;

	fn bitor(self, rhs: u32) -> FileMode {
		FileMode(self.mode_bits() | rhs)
	}
}

impl PartialEq<u32> for FileMode {
	fn eq(&self, mode: &u32) -> bool {
		self.0 == *mode
	}
}

impl PartialEq<FileMode> for u32 {
	fn eq(&self, mode: &FileMode) -> bool {
		*self == mode.0
	}
}

impl fmt::Debug for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{:#o}", self.0)
	}
}

impl fmt::Octal for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::Binary for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::LowerHex for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::UpperHex for FileMode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}
