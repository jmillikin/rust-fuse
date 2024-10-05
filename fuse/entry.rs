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

use crate::internal::fuse_kernel;
use crate::internal::timestamp;
use crate::attributes::Attributes;

/// Cacheable directory entry for a filesystem node.
#[derive(Clone, Copy)]
pub struct Entry {
	raw: fuse_kernel::fuse_entry_out,
}

impl Entry {
	/// Creates a new `Entry` for a node with the given attributes.
	#[inline]
	#[must_use]
	pub fn new(attributes: Attributes) -> Entry {
		Self {
			raw: fuse_kernel::fuse_entry_out {
				nodeid: attributes.raw.ino,
				attr: attributes.raw,
				..fuse_kernel::fuse_entry_out::zeroed()
			},
		}
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref(raw: &fuse_kernel::fuse_entry_out) -> &Self {
		let raw_ptr = raw as *const fuse_kernel::fuse_entry_out;
		&*(raw_ptr.cast::<Entry>())
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref_mut(
		raw: &mut fuse_kernel::fuse_entry_out,
	) -> &mut Self {
		let raw_ptr = raw as *mut fuse_kernel::fuse_entry_out;
		&mut *(raw_ptr.cast::<Entry>())
	}

	#[inline]
	#[must_use]
	pub(crate) fn into_entry_out(self) -> fuse_kernel::fuse_entry_out {
		self.raw
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
	pub fn attributes(&self) -> &Attributes {
		unsafe { Attributes::from_ref(&self.raw.attr) }
	}

	/// Returns a mutable reference to the node attributes for this entry.
	#[inline]
	#[must_use]
	pub fn attributes_mut(&mut self) -> &mut Attributes {
		unsafe { Attributes::from_ref_mut(&mut self.raw.attr) }
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

	#[inline]
	#[must_use]
	pub(crate) fn as_v7p9(&self) -> &fuse_kernel::fuse_entry_out {
		let self_ptr = self as *const Entry;
		unsafe { &*(self_ptr.cast::<fuse_kernel::fuse_entry_out>()) }
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_v7p1(
		&self,
	) -> &[u8; fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE] {
		let self_ptr = self as *const Entry;
		const OUT_SIZE: usize = fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE;
		unsafe { &*(self_ptr.cast::<[u8; OUT_SIZE]>()) }
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
