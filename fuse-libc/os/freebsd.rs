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

use core::ffi;

#[cfg(target_os = "freebsd")]
use fuse::os::freebsd as fuse_os_freebsd;

use crate::io::iovec::IoVec;
use crate::io::socket::{FuseServerSocket, LibcError};

#[cfg(all(doc, not(target_os = "freebsd")))]
mod fuse_os_freebsd {
	#[derive(Copy, Clone)]
	pub struct MountOptions<'a> { _p: &'a () }
}

const MNT_NOSUID: i32 = 0x08;

const DEFAULT_FLAGS: i32 = MNT_NOSUID;

#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	opts: fuse_os_freebsd::MountOptions<'a>,
	flags: i32,
}

impl<'a> MountOptions<'a> {
	#[must_use]
	pub fn flags(&self) -> i32 {
		self.flags
	}

	pub fn set_flags(&mut self, flags: i32) {
		self.flags = flags;
	}
}

impl<'a> From<fuse_os_freebsd::MountOptions<'a>> for MountOptions<'a> {
	fn from(opts: fuse_os_freebsd::MountOptions<'a>) -> Self {
		Self {
			opts,
			flags: DEFAULT_FLAGS,
		}
	}
}

pub fn mount<'a>(
	target: &ffi::CStr,
	options: impl Into<MountOptions<'a>>,
) -> Result<FuseServerSocket, LibcError> {
	let options = options.into();
	let opts = options.opts;
	let socket = FuseServerSocket::new()?;

	let mut fd_buf = [0u8; 32];
	fmt_raw_fd(&mut fd_buf, socket.fuse_device_fd());
	let mut iovecs = [
		// fstype
		IoVec::global(b"fstype\0"),
		IoVec::global(b"fusefs\0"),

		// from
		IoVec::global(b"from\0"),
		IoVec::global(b"/dev/fuse\0"),

		// fspath
		IoVec::global(b"fspath\0"),
		IoVec::borrow(target.to_bytes_with_nul()),

		// fd
		IoVec::global(b"fd\0"),
		IoVec::borrow(&fd_buf),

		// placeholder: default_permissions
		IoVec::null(),
		IoVec::null(),

		// placeholder: subtype=
		IoVec::null(),
		IoVec::null(),
	];

	let mut iovecs_len: usize = 8;

	if opts.default_permissions() {
		iovecs[iovecs_len] = IoVec::global(b"default_permissions\0");
		iovecs[iovecs_len + 1] = IoVec::global(b"\0");
		iovecs_len += 2;
	}

	if let Some(subtype) = opts.subtype() {
		let subtype_bytes = subtype.as_cstr().to_bytes_with_nul();
		iovecs[iovecs_len] = IoVec::global(b"subtype=\0");
		iovecs[iovecs_len + 1] = IoVec::borrow(subtype_bytes);
		iovecs_len += 2;
	}

	let nmount_rc = unsafe {
		libc::nmount(
			iovecs.as_mut_ptr().cast::<libc::iovec>(),
			iovecs_len as libc::c_uint,
			options.flags,
		)
	};
	if nmount_rc == -1 {
		return Err(LibcError::last_os_error());
	}

	Ok(socket)
}

fn fmt_raw_fd(buf: &mut [u8; 32], fd: u32) {
	let buf_ptr = buf.as_mut_ptr().cast::<libc::c_char>();
	let format_ptr = b"%u\0".as_ptr().cast::<libc::c_char>();
	unsafe {
		libc::snprintf(buf_ptr, 32, format_ptr, fd);
	}
}
