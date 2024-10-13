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

//! Filesystem nodes and node attributes.

use core::fmt;
use core::time;

use crate::NodeAttr;
use crate::internal::timestamp;
use crate::kernel;
use crate::server;

/// Cacheable directory entry for a filesystem node.
#[derive(Clone, Copy)]
pub struct Entry {
	raw: kernel::fuse_entry_out,
}

impl Entry {
	/// Creates a new `Entry` for a node with the given attributes.
	#[inline]
	#[must_use]
	pub fn new(node_attr: NodeAttr) -> Entry {
		Self {
			raw: kernel::fuse_entry_out {
				nodeid: node_attr.raw.ino,
				attr: node_attr.raw,
				..kernel::fuse_entry_out::new()
			},
		}
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref(raw: &kernel::fuse_entry_out) -> &Self {
		let raw_ptr = raw as *const kernel::fuse_entry_out;
		&*(raw_ptr.cast::<Entry>())
	}

	/// Returns the raw [`fuse_entry_out`] for the node entry.
	///
	/// [`fuse_entry_out`]: kernel::fuse_entry_out
	#[inline]
	#[must_use]
	pub fn raw(&self) -> &kernel::fuse_entry_out {
		&self.raw
	}

	/// Returns the generation number for this entry.
	#[inline]
	#[must_use]
	pub fn generation(&self) -> u64 {
		self.raw.generation
	}

	/// Sets the generation number for this entry.
	#[inline]
	pub fn set_generation(&mut self, generation: u64) {
		self.raw.generation = generation;
	}

	/// Returns the node attributes for this entry.
	#[inline]
	#[must_use]
	pub fn attributes(&self) -> &NodeAttr {
		unsafe { NodeAttr::from_ref(&self.raw.attr) }
	}

	/// Returns a mutable reference to the node attributes for this entry.
	#[inline]
	#[must_use]
	pub fn attributes_mut(&mut self) -> &mut NodeAttr {
		unsafe { NodeAttr::from_ref_mut(&mut self.raw.attr) }
	}

	/// Returns the lookup cache timeout for this entry.
	#[inline]
	#[must_use]
	pub fn cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.raw.entry_valid, self.raw.entry_valid_nsec)
	}

	/// Sets the lookup cache timeout for this entry.
	#[inline]
	pub fn set_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.raw.entry_valid = seconds;
		self.raw.entry_valid_nsec = nanos;
	}

	/// Returns the attribute cache timeout for this entry.
	#[inline]
	#[must_use]
	pub fn attribute_cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.raw.attr_valid, self.raw.attr_valid_nsec)
	}

	/// Sets the attribute cache timeout for this entry.
	#[inline]
	pub fn set_attribute_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.raw.attr_valid = seconds;
		self.raw.attr_valid_nsec = nanos;
	}
}

impl fmt::Debug for Entry {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Entry")
			.field("generation", &self.generation())
			.field("attributes", &self.attributes())
			.field("cache_timeout", &self.cache_timeout())
			.field("attribute_cache_timeout", &self.attribute_cache_timeout())
			.finish()
	}
}

impl server::FuseReply for Entry {
	#[inline]
	fn send_to<S: server::FuseSocket>(
		&self,
		reply_sender: server::FuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		self.raw.send_to(reply_sender)
	}
}
