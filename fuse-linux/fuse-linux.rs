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

#![no_std]

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

#[cfg(target_os = "linux")]
use fuse::os::linux as fuse_os_linux;

#[cfg(all(doc, not(target_os = "linux")))]
mod fuse_os_linux {
	pub struct MountOptions<'a> { _p: &'a () }
}

mod socket;
mod sys;

pub use socket::{CuseServerSocket, FuseServerSocket};

const MS_NOSUID: u32 = 1 << 1;
const MS_NODEV:  u32 = 1 << 2;

const DEFAULT_FLAGS: u32 = MS_NOSUID | MS_NODEV;

// This is technically incorrect, because Linux can be compiled with
// different page sizes (and often is on e.g. ARM). But we're using this value
// only as a maximum length limit for `mount(2)` data, so hardcoding should
// be fine.
const PAGE_SIZE: usize = 4096;

const DEV_CUSE: &ffi::CStr = c"/dev/cuse";
const DEV_FUSE: &ffi::CStr = c"/dev/fuse";

#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	opts: fuse_os_linux::MountOptions<'a>,
	dev_fuse: Option<&'a ffi::CStr>,
	flags: u32,
}

impl<'a> MountOptions<'a> {
	#[must_use]
	pub fn dev_fuse(&self) -> &'a ffi::CStr {
		self.dev_fuse.unwrap_or(DEV_FUSE)
	}

	pub fn set_dev_fuse(&mut self, dev_fuse: Option<&'a ffi::CStr>) {
		self.dev_fuse = dev_fuse;
	}

	#[must_use]
	pub fn flags(&self) -> u32 {
		self.flags
	}

	pub fn set_flags(&mut self, flags: u32) {
		self.flags = flags;
	}
}

impl<'a> From<fuse_os_linux::MountOptions<'a>> for MountOptions<'a> {
	fn from(opts: fuse_os_linux::MountOptions<'a>) -> Self {
		Self {
			opts,
			dev_fuse: None,
			flags: DEFAULT_FLAGS,
		}
	}
}

pub fn mount<'a>(
	target: &ffi::CStr,
	options: impl Into<MountOptions<'a>>,
) -> Result<FuseServerSocket, linux_errno::Error> {
	use fuse::os::linux::mount_data;

	let options = options.into();
	let mut opts = options.opts;
	if opts.root_mode().is_none() {
		opts.set_root_mode(Some(get_root_mode(target)?));
	}
	if opts.user_id().is_none() {
		opts.set_user_id(Some(sys::getuid()));
	}
	if opts.group_id().is_none() {
		opts.set_group_id(Some(sys::getgid()));
	}

	let socket = FuseServerSocket::open(options.dev_fuse())?;
	opts.set_fuse_device_fd(Some(socket.fuse_device_fd()));

	let mut mount_data_buf = [0u8; PAGE_SIZE];
	let mount_data = match mount_data(&opts, &mut mount_data_buf) {
		Some(mount_data) => mount_data,
		_ => return Err(linux_errno::EINVAL),
	};

	unsafe {
		sys::mount(
			opts.mount_source().as_cstr(),
			target,
			opts.mount_type().as_cstr(),
			options.flags,
			mount_data,
		)?;
	}
	Ok(socket)
}

fn get_root_mode(target: &ffi::CStr) -> Result<fuse::FileMode, linux_errno::Error> {
	let statx = unsafe {
		sys::statx(sys::AT_FDCWD, target, 0, sys::STATX_MODE)?
	};
	Ok(fuse::FileMode::new(u32::from(statx.stx_mode)))
}
