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

use std::slice;

use crate::internal::fuse_io;
use crate::internal::fuse_kernel;

macro_rules! entry_out_methods {
	($field:ident) => {
		pub fn node_id(&self) -> Option<node::NodeId> {
			node::NodeId::new(self.$field.nodeid)
		}

		pub fn set_node_id(&mut self, node_id: node::NodeId) {
			self.$field.attr.ino = node_id.get();
			self.$field.nodeid = node_id.get();
		}

		pub fn generation(&self) -> u64 {
			self.$field.generation
		}

		pub fn set_generation(&mut self, generation: u64) {
			self.$field.generation = generation;
		}

		pub fn cache_duration(&self) -> std::time::Duration {
			std::time::Duration::new(
				self.$field.entry_valid,
				self.$field.entry_valid_nsec,
			)
		}

		pub fn set_cache_duration(
			&mut self,
			cache_duration: std::time::Duration,
		) {
			self.$field.entry_valid = cache_duration.as_secs();
			self.$field.entry_valid_nsec = cache_duration.subsec_nanos();
		}

		pub fn attr(&self) -> &crate::protocol::NodeAttr {
			crate::protocol::NodeAttr::new_ref(&self.$field.attr)
		}

		pub fn attr_mut(&mut self) -> &mut crate::protocol::NodeAttr {
			crate::protocol::NodeAttr::new_ref_mut(&mut self.$field.attr)
		}

		pub fn attr_cache_duration(&self) -> std::time::Duration {
			std::time::Duration::new(
				self.$field.attr_valid,
				self.$field.attr_valid_nsec,
			)
		}

		pub fn set_attr_cache_duration(&mut self, d: std::time::Duration) {
			self.$field.attr_valid = d.as_secs();
			self.$field.attr_valid_nsec = d.subsec_nanos();
		}
	};
}

pub(crate) fn encode_entry_out<'a, Chan: fuse_io::Channel>(
	enc: fuse_io::ResponseEncoder<Chan>,
	raw_entry: &'a fuse_kernel::fuse_entry_out,
) -> std::io::Result<()> {
	// The `fuse_attr::blksize` field was added in FUSE v7.9.
	if enc.version().minor() < 9 {
		let buf: &[u8] = unsafe {
			slice::from_raw_parts(
				(raw_entry as *const fuse_kernel::fuse_entry_out) as *const u8,
				fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE,
			)
		};
		return enc.encode_bytes(buf);
	}

	enc.encode_sized(raw_entry)
}

pub(crate) fn encode_entry_sized<'a, Chan: fuse_io::Channel, T: Sized>(
	enc: fuse_io::ResponseEncoder<Chan>,
	raw_entry: &'a fuse_kernel::fuse_entry_out,
	t: &T,
) -> std::io::Result<()> {
	// The `fuse_attr::blksize` field was added in FUSE v7.9.
	if enc.version().minor() < 9 {
		let buf: &[u8] = unsafe {
			slice::from_raw_parts(
				(raw_entry as *const fuse_kernel::fuse_entry_out) as *const u8,
				fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE,
			)
		};
		return enc.encode_sized_bytes(buf, t);
	}

	enc.encode_sized_sized(raw_entry, t)
}

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
