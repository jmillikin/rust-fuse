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

use core::ffi;

use fuse::node;
#[cfg(target_os = "linux")]
use fuse::os::linux as fuse_os_linux;

use crate::io::socket::{FuseServerSocket, LibcError};

#[cfg(all(doc, not(target_os = "linux")))]
mod fuse_os_linux {
	pub struct MountOptions<'a> { _p: &'a () }
}

const MS_NOSUID: u32 = 0x2;
const MS_NODEV: u32 = 0x4;

const DEFAULT_FLAGS: u32 = MS_NOSUID | MS_NODEV;

// This is technically incorrect, because Linux can be compiled with
// different page sizes (and often is on e.g. ARM). But we're using this value
// only as a maximum length limit for `mount(2)` data, so hardcoding should
// be fine.
const PAGE_SIZE: usize = 4096;

#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	opts: fuse_os_linux::MountOptions<'a>,
	dev_fuse: Option<&'a ffi::CStr>,
	flags: u32,
}

impl<'a> MountOptions<'a> {
	#[must_use]
	pub fn dev_fuse(&self) -> &'a ffi::CStr {
		self.dev_fuse.unwrap_or(crate::DEV_FUSE)
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
) -> Result<FuseServerSocket, LibcError> {
	use fuse::os::linux::mount_data;

	let options = options.into();
	let mut opts = options.opts;
	if opts.root_mode().is_none() {
		opts.set_root_mode(Some(get_root_mode(target)?));
	}
	if opts.user_id().is_none() {
		opts.set_user_id(Some(unsafe { libc::getuid() }));
	}
	if opts.group_id().is_none() {
		opts.set_group_id(Some(unsafe { libc::getgid() }));
	}

	let socket = FuseServerSocket::open(options.dev_fuse())?;
	opts.set_fuse_device_fd(Some(socket.fuse_device_fd()));

	let mut mount_data_buf = [0u8; PAGE_SIZE];
	let mount_data = match mount_data(&opts, &mut mount_data_buf) {
		Some(mount_data) => mount_data,
		_ => return Err(LibcError::from_raw_os_error(libc::EINVAL)),
	};

	let rc = unsafe {
		libc::mount(
			opts.mount_source().as_cstr().as_ptr(),
			target.as_ptr(),
			opts.mount_type().as_cstr().as_ptr(),
			options.flags as libc::c_ulong,
			mount_data.as_ptr().cast(),
		)
	};
	if rc != 0 {
		return Err(LibcError::last_os_error());
	}
	Ok(socket)
}

fn get_root_mode(target: &ffi::CStr) -> Result<node::Mode, LibcError> {
	let mut statx_buf: libc::statx = unsafe { core::mem::zeroed() };
	let rc = unsafe {
		libc::statx(
			libc::AT_FDCWD,
			target.as_ptr(),
			0,
			libc::STATX_MODE,
			&mut statx_buf as *mut libc::statx,
		)
	};
	if rc != 0 {
		return Err(LibcError::last_os_error());
	}
	Ok(node::Mode::new(u32::from(statx_buf.stx_mode)))
}
