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

use core::{fmt, num};

use crate::internal::fuse_kernel;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct NodeId(num::NonZeroU64);

pub const ROOT_ID: NodeId = NodeId(unsafe {
	num::NonZeroU64::new_unchecked(fuse_kernel::FUSE_ROOT_ID)
});

impl NodeId {
	pub fn new(id: u64) -> Option<NodeId> {
		num::NonZeroU64::new(id).map(|bits| NodeId(bits))
	}

	pub(crate) unsafe fn new_unchecked(id: u64) -> NodeId {
		NodeId(num::NonZeroU64::new_unchecked(id))
	}

	pub fn get(&self) -> u64 {
		self.0.get()
	}
}

impl fmt::Debug for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::Display for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::Binary for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::LowerHex for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::UpperHex for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}
