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

use crate::internal::fuse_kernel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolVersion {
	major: u32,
	minor: u32,
}

impl ProtocolVersion {
	pub(crate) const LATEST: ProtocolVersion = ProtocolVersion {
		major: fuse_kernel::FUSE_KERNEL_VERSION,
		minor: fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
	};

	pub fn new(major: u32, minor: u32) -> ProtocolVersion {
		ProtocolVersion { major, minor }
	}

	pub fn major(&self) -> u32 {
		self.major
	}

	pub fn minor(&self) -> u32 {
		self.minor
	}
}
