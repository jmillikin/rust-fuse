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

use crate::protocol::node;
use crate::protocol::prelude::*;

#[cfg(test)]
mod forget_test;

// ForgetRequest {{{

/// **\[UNSTABLE\]**
pub struct ForgetRequest<'a> {
	single: Option<ForgetNode>,
	batch: &'a [ForgetNode],
}

impl ForgetRequest<'_> {
	pub fn nodes(&self) -> &[ForgetNode] {
		if let Some(ref n) = self.single {
			return slice::from_ref(n);
		}
		return self.batch;
	}
}

/// **\[UNSTABLE\]**
#[repr(C)]
#[derive(Debug)]
pub struct ForgetNode {
	id: Option<node::NodeId>,
	count: u64,
}

impl ForgetNode {
	pub fn id(&self) -> Option<node::NodeId> {
		self.id
	}

	pub fn count(&self) -> u64 {
		self.count
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ForgetRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		if header.opcode == fuse_kernel::FUSE_BATCH_FORGET {
			let raw: &fuse_kernel::fuse_batch_forget_in = dec.next_sized()?;
			let batch_size =
				raw.count * size_of::<fuse_kernel::fuse_forget_one>() as u32;
			let batch_bytes = dec.next_bytes(batch_size)?;
			let batch = unsafe {
				slice::from_raw_parts(
					batch_bytes.as_ptr() as *const ForgetNode,
					raw.count as usize,
				)
			};
			return Ok(Self {
				single: None,
				batch,
			});
		}

		debug_assert!(header.opcode == fuse_kernel::FUSE_FORGET);
		let raw: &fuse_kernel::fuse_forget_in = dec.next_sized()?;
		let node_id = try_node_id(header.nodeid)?;
		Ok(Self {
			single: Some(ForgetNode {
				id: Some(node_id),
				count: raw.nlookup,
			}),
			batch: &[],
		})
	}
}

// }}}
