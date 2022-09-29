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

use crate::internal::fuse_kernel;

#[derive(Clone, Copy)]
pub(crate) struct Versioned<U> {
	version_minor: u32,
	u: U,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_read_in<'a> {
	v7p1: &'a fuse_read_in_v7p1,
	v7p9: &'a fuse_kernel::fuse_read_in,
}

#[repr(C)]
pub(crate) struct fuse_read_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) offset: u64,
	pub(crate) size: u32,
	pub(crate) padding: u32,
}

impl<'a> Versioned<fuse_read_in<'a>> {
	#[inline]
	pub(crate) fn new_v7p1(
		version_minor: u32,
		v7p1: &'a fuse_read_in_v7p1,
	) -> Self {
		Self {
			version_minor,
			u: fuse_read_in { v7p1 },
		}
	}

	#[inline]
	pub(crate) fn new_v7p9(
		version_minor: u32,
		v7p9: &'a fuse_kernel::fuse_read_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_read_in { v7p9 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p1(self) -> &'a fuse_read_in_v7p1 {
		unsafe { self.u.v7p1 }
	}

	#[inline]
	pub(crate) fn as_v7p9(self) -> Option<&'a fuse_kernel::fuse_read_in> {
		if self.version_minor >= 9 {
			return Some(unsafe { self.u.v7p9 });
		}
		None
	}
}
