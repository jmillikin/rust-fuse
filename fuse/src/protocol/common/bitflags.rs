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
			$item_mask:ident: {
				get: $item_getter:ident ,
				set: $item_setter:ident ,
			},
		)*
	) => {
		$( #[$struct_doc] )*
		#[derive(Copy, Clone, PartialEq, Eq)]
		#[repr(transparent)]
		pub struct $struct_name(u32);

		impl $struct_name {
			pub fn new() -> Self {
				Self(0)
			}

			fn field_name(mask: u32) -> Option<&'static str> {
				match mask {
					$(
						fuse_kernel::$item_mask => Some(stringify!($item_getter)),
					)*
					_ => None,
				}
			}

			$(
				$( #[$item_doc] )*
				pub fn $item_getter(&self) -> bool {
					(self.0 & fuse_kernel::$item_mask) > 0
				}

				pub fn $item_setter(&mut self, $item_getter: bool) {
					if $item_getter {
						self.0 |= fuse_kernel::$item_mask;
					} else {
						self.0 &= !fuse_kernel::$item_mask;
					}
				}
			)*
		}

		impl fmt::Debug for $struct_name {
			fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
				let mut out = fmt.debug_struct(stringify!($struct_name));
				for off in 0..32 {
					let mask: u32 = 1 << off;
					let is_set = self.0 & mask > 0;
					match Self::field_name(mask) {
						Some(name) => {
							out.field(name, &is_set);
						},
						None => {
							if is_set {
								out.field(&format!("{:#010X}", mask), &is_set);
							}
						},
					}
				}
				out.finish()
			}
		}
	}
}
