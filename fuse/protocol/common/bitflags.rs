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

macro_rules! bitflags_struct {
	(
		$( #[$struct_doc:meta] )*
		pub struct $struct_name:ident(u32);

		$(
			$( #[$item_doc:meta] )*
			$item_mask:path: $item_name:ident ,
		)*
	) => {
		$( #[$struct_doc] )*
		#[non_exhaustive]
		#[derive(Copy, Clone, PartialEq, Eq)]
		pub struct $struct_name {
			bits: u32,
			$(
				$( #[$item_doc] )*
				pub $item_name: bool,
			)*
		}

		impl $struct_name {
			#[allow(dead_code)]
			pub fn new() -> $struct_name {
				Self {
					bits: 0,
					$(
						$item_name: false,
					)*
				}
			}

			fn known_field(&self, mask: u32) -> Option<(&'static str, bool)> {
				match mask {
					$(
						$item_mask => Some((
							stringify!($item_name),
							self.$item_name,
						)),
					)*
					_ => None,
				}
			}

			#[allow(dead_code)]
			fn from_bits(bits: u32) -> $struct_name {
				Self {
					bits,
					$(
						$item_name: (bits & $item_mask) > 0,
					)*
				}
			}

			#[allow(dead_code)]
			fn to_bits(&self) -> u32 {
				let mut out = 0;
				$(
					if self.$item_name {
						out |= $item_mask;
					}
				)*
				out | self.bits
			}
		}

		impl fmt::Debug for $struct_name {
			fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
				let mut out = fmt.debug_struct(stringify!($struct_name));
				for off in 0..32 {
					let mask: u32 = 1 << off;
					match self.known_field(mask) {
						Some((name, is_set)) => {
							out.field(name, &is_set);
						},
						None => {
							let is_set = self.bits & mask > 0;
							if is_set {
								// TODO: support no_std
								#[cfg(feature = "std")]
								out.field(&format!("{:#010X}", mask), &is_set);
								#[cfg(not(feature = "std"))]
								out.field("[unknown]", &is_set);
							}
						},
					}
				}
				out.finish()
			}
		}
	}
}
