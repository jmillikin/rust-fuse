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

use core::{fmt, slice, time};

use crate::internal::fuse_io;
use crate::internal::fuse_kernel;
use crate::protocol::common::{NodeAttr, NodeId};

pub struct Node(fuse_kernel::fuse_entry_out);

impl Node {
	pub fn id(&self) -> Option<NodeId> {
		NodeId::new(self.0.nodeid)
	}

	pub fn set_id(&mut self, id: NodeId) {
		self.0.nodeid = id.get();
	}

	pub fn generation(&self) -> u64 {
		self.0.generation
	}

	pub fn set_generation(&mut self, generation: u64) {
		self.0.generation = generation;
	}

	pub fn cache_timeout(&self) -> time::Duration {
		time::Duration::new(self.0.entry_valid, self.0.entry_valid_nsec)
	}

	pub fn set_cache_timeout(&mut self, d: time::Duration) {
		self.0.entry_valid = d.as_secs();
		self.0.entry_valid_nsec = d.subsec_nanos();
	}

	pub fn attr_cache_timeout(&self) -> time::Duration {
		time::Duration::new(self.0.attr_valid, self.0.attr_valid_nsec)
	}

	pub fn set_attr_cache_timeout(&mut self, d: time::Duration) {
		self.0.attr_valid = d.as_secs();
		self.0.attr_valid_nsec = d.subsec_nanos();
	}

	pub fn attr(&self) -> &NodeAttr {
		NodeAttr::new_ref(&self.0.attr)
	}

	pub fn attr_mut(&mut self) -> &mut NodeAttr {
		NodeAttr::new_ref_mut(&mut self.0.attr)
	}

	pub(crate) fn new_ref(raw: &fuse_kernel::fuse_entry_out) -> &Node {
		unsafe { &*(raw as *const fuse_kernel::fuse_entry_out as *const Node) }
	}

	pub(crate) fn new_ref_mut(
		raw: &mut fuse_kernel::fuse_entry_out,
	) -> &mut Node {
		unsafe { &mut *(raw as *mut fuse_kernel::fuse_entry_out as *mut Node) }
	}
}

impl fmt::Debug for Node {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Node")
			.field("id", &format_args!("{:?}", &self.id()))
			.field("generation", &self.generation())
			.field("cache_timeout", &self.cache_timeout())
			.field("attr_cache_timeout", &self.attr_cache_timeout())
			.field("attr", self.attr())
			.finish()
	}
}

impl Node {
	pub(crate) fn encode_entry<'a, Chan: fuse_io::Channel>(
		&self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		// The `fuse_attr::blksize` field was added in FUSE v7.9.
		if enc.version().minor() < 9 {
			let buf: &[u8] = unsafe {
				slice::from_raw_parts(
					(&self.0 as *const fuse_kernel::fuse_entry_out)
						as *const u8,
					fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE,
				)
			};
			return enc.encode_bytes(buf);
		}

		enc.encode_sized(&self.0)
	}

	#[cfg(feature = "unstable_create")]
	pub(crate) fn encode_entry_sized<'a, Chan: fuse_io::Channel, T: Sized>(
		&self,
		enc: fuse_io::ResponseEncoder<Chan>,
		t: &T,
	) -> Result<(), Chan::Error> {
		// The `fuse_attr::blksize` field was added in FUSE v7.9.
		if enc.version().minor() < 9 {
			let buf: &[u8] = unsafe {
				slice::from_raw_parts(
					(&self.0 as *const fuse_kernel::fuse_entry_out)
						as *const u8,
					fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE,
				)
			};
			return enc.encode_sized_bytes(buf, t);
		}

		enc.encode_sized_sized(&self.0, t)
	}
}
