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

macro_rules! bitflags {
	($flag:ident, $flags:ident, $bits:ident, {
		$(
			$( #[$item_doc:meta] )*
			$item_pub_name:ident = fuse_kernel::$item_name:ident ;
		)*
	}) => {
		use super::{$flag, $flags};

		$(
			$( #[$item_doc] )*
			const $item_name: $flag = $flag::new(fuse_kernel::$item_name as $bits);
		)*

		impl $flag {
			$(
				$( #[$item_doc] )*
				pub const $item_pub_name: $flag = $item_name;
			)*

			#[allow(dead_code)]
			#[inline(always)]
			#[must_use]
			const fn new(mask: $bits) -> Self {
				Self { mask }
			}

			#[must_use]
			const fn flag_name(mask: $bits) -> Option<&'static str> {
				match ($flag { mask }) {
				$(
					$item_name => Some(stringify!($item_pub_name)),
				)*
				_ => None,
				}
			}
		}

		impl core::fmt::Debug for $flag {
			fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
				match Self::flag_name(self.mask) {
					Some(name) => fmt.write_str(&name),
					None => write!(fmt, bitflags_bits_fmt!($bits), self.mask),
				}
			}
		}

		impl core::fmt::Debug for $flags {
			fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
				fmt.write_str(stringify!($flags))?;
				fmt.write_str(" ")?;
				let mut dbg = fmt.debug_set();
				for off in bitflags_bits_range!($bits) {
					let mask: $bits = 1 << off;
					if self.bits & mask == 0 {
						continue;
					}
					dbg.entry(&$flag { mask });
				}
				dbg.finish()
			}
		}

		#[allow(dead_code)]
		impl $flags {
			#[inline(always)]
			#[must_use]
			pub const fn new() -> Self {
				Self { bits: 0 }
			}

			#[inline(always)]
			#[must_use]
			pub(crate) fn reborrow_mut<'a>(r: &'a mut $bits) -> &'a mut Self {
				let ptr = (r as *mut $bits).cast::<Self>();
				unsafe { &mut *ptr }
			}

			#[inline(always)]
			#[must_use]
			pub const fn get(&self, flag: $flag) -> bool {
				self.bits & flag.mask == flag.mask
			}

			#[inline(always)]
			pub fn set(&mut self, flag: $flag) {
				self.bits |= flag.mask;
			}

			#[inline]
			pub(crate) fn set_to(&mut self, flag: $flag, set: bool) {
				if set {
					self.bits |= flag.mask;
				} else {
					self.bits &= !flag.mask;
				}
			}
		}

		impl core::cmp::PartialEq<$flag> for $flags {
			fn eq(&self, rhs: &$flag) -> bool {
				self.bits == rhs.mask
			}
		}

		impl core::cmp::PartialEq<$flags> for $flag {
			fn eq(&self, rhs: &$flags) -> bool {
				self.mask == rhs.bits
			}
		}

		impl core::ops::BitOr<$flag> for $flag {
			type Output = $flags;
			fn bitor(self, rhs: $flag) -> $flags {
				$flags { bits: self.mask | rhs.mask }
			}
		}

		impl core::ops::BitOr<$flag> for $flags {
			type Output = $flags;
			fn bitor(self, rhs: $flag) -> $flags {
				$flags { bits: self.bits | rhs.mask }
			}
		}

		impl core::ops::BitOr<$flags> for $flag {
			type Output = $flags;
			fn bitor(self, rhs: $flags) -> $flags {
				$flags { bits: self.mask | rhs.bits }
			}
		}

		impl core::ops::BitOr<$flags> for $flags {
			type Output = $flags;
			fn bitor(self, rhs: $flags) -> $flags {
				$flags { bits: self.bits | rhs.bits }
			}
		}

		impl core::ops::BitOrAssign<$flag> for $flags {
			fn bitor_assign(&mut self, rhs: $flag) {
				self.bits = self.bits | rhs.mask;
			}
		}

		impl core::ops::BitOrAssign<$flags> for $flags {
			fn bitor_assign(&mut self, rhs: $flags) {
				self.bits = self.bits | rhs.bits;
			}
		}

		impl core::ops::BitAnd<$flags> for $flags {
			type Output = $flags;
			fn bitand(self, rhs: $flags) -> $flags {
				$flags { bits: self.bits & rhs.bits}
			}
		}

		impl core::ops::BitAndAssign<$flags> for $flags {
			fn bitand_assign(&mut self, rhs: $flags) {
				self.bits = self.bits & rhs.bits;
			}
		}
	};
}

macro_rules! bitflags_bits_range {
	(u32) => { 0..32 };
	(u64) => { 0..64 };
}

macro_rules! bitflags_bits_fmt {
	(u32) => { "{:#010X}" };
	(u64) => { "{:#018X}" };
}
