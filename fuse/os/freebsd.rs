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

use core::ffi;

// MountOptions {{{

#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	default_permissions: bool,
	fs_subtype: Option<&'a ffi::CStr>,
}

impl<'a> MountOptions<'a> {
	#[must_use]
	pub fn new() -> Self {
		MountOptions {
			default_permissions: false,
			fs_subtype: None,
		}
	}

	#[must_use]
	pub fn default_permissions(&self) -> bool {
		self.default_permissions
	}

	pub fn set_default_permissions(&mut self, default_permissions: bool) {
		self.default_permissions = default_permissions;
	}

	#[must_use]
	pub fn fs_subtype(&self) -> Option<&'a ffi::CStr> {
		self.fs_subtype
	}

	pub fn set_fs_subtype(&mut self, fs_subtype: Option<&'a ffi::CStr>) {
		self.fs_subtype = fs_subtype;
	}
}

// }}}
