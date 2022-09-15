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

//#![cfg_attr(not(any(doc, feature = "std")), no_std)]

// use core::ffi::CStr;
use std::ffi::CStr;

mod io {
	pub(crate) mod iovec;
	pub(crate) mod stream;
}

pub use crate::io::stream::{FuseStream, LibcError};

#[cfg(any(doc, not(target_os = "freebsd")))]
pub use crate::io::stream::CuseStream;

pub mod os {
	#[cfg(any(doc, target_os = "freebsd"))]
	pub mod freebsd;

	#[cfg(any(doc, target_os = "linux"))]
	pub mod linux;
}

#[cfg(not(target_os = "freebsd"))]
const DEV_CUSE: &'static CStr = unsafe {
	CStr::from_bytes_with_nul_unchecked(b"/dev/cuse\0")
};

const DEV_FUSE: &'static CStr = unsafe {
	CStr::from_bytes_with_nul_unchecked(b"/dev/fuse\0")
};
