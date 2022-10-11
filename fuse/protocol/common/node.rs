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

use crate::node;
use crate::internal::fuse_kernel;
use crate::internal::timestamp;
use crate::protocol::common::NodeAttr;
use crate::server::encode;

pub struct Node(fuse_kernel::fuse_entry_out);

impl Node {
	#[must_use]
	pub fn id(&self) -> Option<node::Id> {
		node::Id::new(self.0.nodeid)
	}

	pub fn set_id(&mut self, id: node::Id) {
		self.0.nodeid = id.get();
	}

	#[must_use]
	pub fn generation(&self) -> u64 {
		self.0.generation
	}

	pub fn set_generation(&mut self, generation: u64) {
		self.0.generation = generation;
	}

	#[must_use]
	pub fn cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.0.entry_valid, self.0.entry_valid_nsec)
	}

	pub fn set_cache_timeout(&mut self, d: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(d);
		self.0.entry_valid = seconds;
		self.0.entry_valid_nsec = nanos;
	}

	#[must_use]
	pub fn attr_cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.0.attr_valid, self.0.attr_valid_nsec)
	}

	pub fn set_attr_cache_timeout(&mut self, d: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(d);
		self.0.attr_valid = seconds;
		self.0.attr_valid_nsec = nanos;
	}

	#[must_use]
	pub fn attr(&self) -> &NodeAttr {
		NodeAttr::new_ref(&self.0.attr)
	}

	#[must_use]
	pub fn attr_mut(&mut self) -> &mut NodeAttr {
		NodeAttr::new_ref_mut(&mut self.0.attr)
	}

	pub(crate) fn new_ref(raw: &fuse_kernel::fuse_entry_out) -> &Node {
		let p = (raw as *const fuse_kernel::fuse_entry_out).cast::<Node>();
		unsafe { &*p }
	}

	pub(crate) fn new_ref_mut(
		raw: &mut fuse_kernel::fuse_entry_out,
	) -> &mut Node {
		let p = (raw as *mut fuse_kernel::fuse_entry_out).cast::<Node>();
		unsafe { &mut *p }
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
	pub(crate) fn encode_entry<'a, S: encode::SendOnce>(
		&'a self,
		enc: encode::ReplyEncoder<S>,
		version_minor: u32,
	) -> S::Result {
		// The `fuse_attr::blksize` field was added in FUSE v7.9.
		if version_minor < 9 {
			let buf: &'a [u8] = unsafe {
				let raw_ptr = &self.0 as *const fuse_kernel::fuse_entry_out;
				slice::from_raw_parts(
					raw_ptr.cast::<u8>(),
					fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE,
				)
			};
			return enc.encode_bytes(buf);
		}

		enc.encode_sized(&self.0)
	}

	pub(crate) fn encode_entry_sized<'a, S: encode::SendOnce, T: Sized>(
		&'a self,
		enc: encode::ReplyEncoder<S>,
		version_minor: u32,
		t: &T,
	) -> S::Result {
		// The `fuse_attr::blksize` field was added in FUSE v7.9.
		if version_minor < 9 {
			let buf: &'a [u8] = unsafe {
				let raw_ptr = &self.0 as *const fuse_kernel::fuse_entry_out;
				slice::from_raw_parts(
					raw_ptr.cast::<u8>(),
					fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE,
				)
			};
			return enc.encode_sized_bytes(buf, t);
		}

		enc.encode_sized_sized(&self.0, t)
	}
}
