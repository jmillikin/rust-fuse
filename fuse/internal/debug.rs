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

use core::ascii;
use core::fmt;

#[inline]
#[must_use]
pub(crate) fn hex_u32(value: u32) -> impl fmt::Debug {
	DebugHexU32(value)
}

struct DebugHexU32(u32);

impl fmt::Debug for DebugHexU32 {
	#[inline]
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		// 8 hex digits + 2 for leading "0x".
		write!(fmt, "{:#010X}", self.0)
	}
}

#[inline]
#[must_use]
pub(crate) fn hex_u64(value: u64) -> impl fmt::Debug {
	DebugHexU64(value)
}

struct DebugHexU64(u64);

impl fmt::Debug for DebugHexU64 {
	#[inline]
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		// 16 hex digits + 2 for leading "0x".
		write!(fmt, "{:#018X}", self.0)
	}
}

#[inline]
#[must_use]
pub(crate) fn bytes<'a>(value: &'a [u8]) -> impl fmt::Debug + 'a {
	DebugBytesAsString(value)
}

struct DebugBytesAsString<'a>(&'a [u8]);

impl fmt::Debug for DebugBytesAsString<'_> {
	#[inline]
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "\"")?;
		for byte in self.0.iter().flat_map(|&b| ascii::escape_default(b)) {
			fmt::Write::write_char(fmt, byte as char)?;
		}
		write!(fmt, "\"")
	}
}
