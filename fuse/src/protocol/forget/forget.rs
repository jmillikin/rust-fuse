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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "forget_test")]
mod forget_test;

// ForgetRequest {{{

#[derive(Debug)]
pub struct ForgetRequestItem {
	node_id: NodeId,
	lookup_count: u64,
}

impl ForgetRequestItem {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn lookup_count(&self) -> u64 {
		self.lookup_count
	}
}

/// Request type for [`FuseHandlers::forget`].
///
/// [`FuseHandlers::forget`]: ../../trait.FuseHandlers.html#method.forget
pub struct ForgetRequest<'a> {
	forget: Option<fuse_kernel::fuse_forget_one>,
	batch_forgets: &'a [fuse_kernel::fuse_forget_one],
}

impl<'a> ForgetRequest<'a> {
	pub fn items(&self) -> impl Iterator<Item = ForgetRequestItem> + 'a {
		self.items_impl()
	}

	fn items_impl(&self) -> ForgetRequestIter<'a> {
		match self.forget {
			Some(item) => ForgetRequestIter::One(Some(item)),
			None => ForgetRequestIter::Batch(self.batch_forgets),
		}
	}
}

impl fmt::Debug for ForgetRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ForgetRequest")
			.field("items", &self.items_impl())
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for ForgetRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		let header = buf.header();
		let mut dec = decode::RequestDecoder::new(buf);
		if header.opcode == fuse_kernel::FUSE_BATCH_FORGET {
			let raw: &'a fuse_kernel::fuse_batch_forget_in =
				dec.next_sized()?;
			let batch_size =
				raw.count * size_of::<fuse_kernel::fuse_forget_one>() as u32;
			let batch_bytes = dec.next_bytes(batch_size)?;
			let batch_forgets: &'a [fuse_kernel::fuse_forget_one] = unsafe {
				slice::from_raw_parts(
					batch_bytes.as_ptr() as *const fuse_kernel::fuse_forget_one,
					raw.count as usize,
				)
			};
			return Ok(Self {
				forget: None,
				batch_forgets,
			});
		}

		debug_assert!(header.opcode == fuse_kernel::FUSE_FORGET);
		let raw: &fuse_kernel::fuse_forget_in = dec.next_sized()?;
		Ok(Self {
			forget: Some(fuse_kernel::fuse_forget_one {
				nodeid: header.nodeid,
				nlookup: raw.nlookup,
			}),
			batch_forgets: &[],
		})
	}
}

// }}}

// ForgetRequestIter {{{

enum ForgetRequestIter<'a> {
	One(Option<fuse_kernel::fuse_forget_one>),
	Batch(&'a [fuse_kernel::fuse_forget_one]),
}

impl ForgetRequestIter<'_> {
	fn clone(&self) -> Self {
		match self {
			Self::One(x) => Self::One(*x),
			Self::Batch(x) => Self::Batch(x),
		}
	}
}

impl Iterator for ForgetRequestIter<'_> {
	type Item = ForgetRequestItem;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(None) => return None,
			Self::One(Some(item)) => {
				let item = *item;
				*self = Self::One(None);
				match NodeId::new(item.nodeid) {
					Some(node_id) => {
						return Some(ForgetRequestItem {
							node_id,
							lookup_count: item.nlookup,
						});
					},
					None => return None,
				};
			},
			Self::Batch(items) => {
				let (head, tail) = next_batch_item(items);
				*self = Self::Batch(tail);
				return head;
			},
		}
	}
}

fn next_batch_item(
	mut items: &[fuse_kernel::fuse_forget_one],
) -> (Option<ForgetRequestItem>, &[fuse_kernel::fuse_forget_one]) {
	loop {
		match items.split_first() {
			None => return (None, &[]),
			Some((head, tail)) => match NodeId::new(head.nodeid) {
				None => {
					items = tail;
				},
				Some(node_id) => {
					let next = Some(ForgetRequestItem {
						node_id,
						lookup_count: head.nlookup,
					});
					return (next, tail);
				},
			},
		}
	}
}

impl fmt::Debug for ForgetRequestIter<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_list().entries(self.clone()).finish()
	}
}

// }}}
