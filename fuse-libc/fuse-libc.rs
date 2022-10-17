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

#![cfg_attr(not(any(doc, feature = "std")), no_std)]

#![warn(
	// API hygiene
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::must_use_candidate,

	// Panic hygiene
	clippy::expect_used,
	clippy::todo,
	clippy::unimplemented,
	clippy::unwrap_used,

	// no_std hygiene
	clippy::std_instead_of_core,

	// Explicit casts
	clippy::fn_to_numeric_cast_any,
	clippy::ptr_as_ptr,

	// Optimization
	clippy::trivially_copy_pass_by_ref,

	// Unused symbols
	clippy::let_underscore_must_use,
	clippy::no_effect_underscore_binding,
	clippy::used_underscore_binding,

	// Leftover debugging
	clippy::print_stderr,
	clippy::print_stdout,
)]

use core::ffi;

mod io {
	pub(crate) mod iovec;
	pub(crate) mod socket;
}

pub use crate::io::socket::{
	FuseServerSocket,
	LibcError,
};

#[cfg(any(doc, not(target_os = "freebsd")))]
pub use crate::io::socket::CuseServerSocket;

pub mod os {
	#[cfg(any(doc, target_os = "freebsd"))]
	pub mod freebsd;

	#[cfg(any(doc, target_os = "linux"))]
	pub mod linux;
}

#[cfg(not(target_os = "freebsd"))]
const DEV_CUSE: &ffi::CStr = unsafe {
	ffi::CStr::from_bytes_with_nul_unchecked(b"/dev/cuse\0")
};

const DEV_FUSE: &ffi::CStr = unsafe {
	ffi::CStr::from_bytes_with_nul_unchecked(b"/dev/fuse\0")
};
