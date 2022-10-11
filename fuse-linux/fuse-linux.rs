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

// use core::ffi::CStr;
use std::ffi::CStr;

use fuse::os::linux::MountData;

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

const DEV_CUSE: &CStr = unsafe {
	CStr::from_bytes_with_nul_unchecked(b"/dev/cuse\0")
};
const DEV_FUSE: &CStr = unsafe {
	CStr::from_bytes_with_nul_unchecked(b"/dev/fuse\0")
};

#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	opts: fuse::os::linux::MountOptions<'a>,
	dev_fuse: Option<&'a CStr>,
	flags: u32,
}

impl<'a> MountOptions<'a> {
	#[must_use]
	pub fn dev_fuse(&self) -> &'a CStr {
		self.dev_fuse.unwrap_or(DEV_FUSE)
	}

	pub fn set_dev_fuse(&mut self, dev_fuse: Option<&'a CStr>) {
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

impl<'a> From<fuse::os::linux::MountOptions<'a>> for MountOptions<'a> {
	fn from(opts: fuse::os::linux::MountOptions<'a>) -> Self {
		Self {
			opts,
			dev_fuse: None,
			flags: DEFAULT_FLAGS,
		}
	}
}

pub fn mount<'a>(
	target: &CStr,
	options: impl Into<MountOptions<'a>>,
) -> Result<FuseServerSocket, linux_errno::Error> {
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
	let mount_data = match MountData::new(&mut mount_data_buf, &opts) {
		Some(mount_data) => mount_data,
		_ => return Err(linux_errno::EINVAL),
	};

	unsafe {
		sys::mount(
			opts.source(),
			target,
			opts.fs_type(),
			options.flags,
			mount_data.as_bytes_with_nul(),
		)?;
	}
	Ok(socket)
}

fn get_root_mode(target: &CStr) -> Result<u32, linux_errno::Error> {
	let statx = unsafe {
		sys::statx(sys::AT_FDCWD, target, 0, sys::STATX_MODE)?
	};
	Ok(u32::from(statx.stx_mode))
}
