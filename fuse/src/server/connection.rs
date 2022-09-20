// Copyright 2021 John Millikin and the rust-fuse contributors.
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

use core::cmp::min;

use crate::internal::fuse_kernel;

pub(super) fn negotiate_version(
	kernel: crate::Version,
) -> Option<crate::Version> {
	if kernel.major() != fuse_kernel::FUSE_KERNEL_VERSION {
		// TODO: hard error on kernel major version < FUSE_KERNEL_VERSION
		return None;
	}
	Some(crate::Version::new(
		fuse_kernel::FUSE_KERNEL_VERSION,
		min(kernel.minor(), fuse_kernel::FUSE_KERNEL_MINOR_VERSION),
	))
}
