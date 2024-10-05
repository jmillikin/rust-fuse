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

use core::fmt;
use core::num::NonZeroU64;

use crate::internal::fuse_kernel;

/// Node IDs are per-mount unique identifiers for filesystem nodes.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId {
	bits: NonZeroU64,
}

const FUSE_ROOT_ID: NodeId = unsafe {
	NodeId::new_unchecked(fuse_kernel::FUSE_ROOT_ID)
};

impl NodeId {
	/// The node ID of the root directory.
	pub const ROOT: NodeId = FUSE_ROOT_ID;

	/// Creates a new `NodeId` if the given value is not zero.
	#[inline]
	#[must_use]
	pub const fn new(id: u64) -> Option<NodeId> {
		match NonZeroU64::new(id) {
			Some(id) => Some(NodeId { bits: id }),
			None => None,
		}
	}

	/// Creates a new `NodeId` without checking that the given value is non-zero.
	///
	/// # Safety
	///
	/// The value must not be zero.
	///
	/// The `NodeId` struct is a wrapper around [`NonZeroU64`], so passing
	/// zero to this function is undefined behavior.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(id: u64) -> NodeId {
		Self { bits: NonZeroU64::new_unchecked(id) }
	}

	/// Returns the node ID as a primitive integer.
	#[inline]
	#[must_use]
	pub const fn get(&self) -> u64 {
		self.bits.get()
	}

	/// Returns whether the node ID is [`NodeId::ROOT`].
	#[inline]
	#[must_use]
	pub const fn is_root(&self) -> bool {
		self.bits.get() == fuse_kernel::FUSE_ROOT_ID
	}
}

impl fmt::Debug for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::Binary for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}
