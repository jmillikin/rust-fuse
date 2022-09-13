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

use crate::io::iovec::IoVec;
use crate::io::stream::{LibcError, LibcStream};

#[derive(Debug)]
pub enum MountError<IoError> {
	InvalidTargetPath,
	InvalidMountSubtype,
	Other(IoError),
}

pub struct LibcFuseMounter<'a> {
	default_permissions: bool,
	mount_flags: i32,
	subtype: Option<&'a [u8]>,
}

impl<'a> LibcFuseMounter<'a> {
	pub fn new() -> Self {
		Self {
			default_permissions: false,
			mount_flags: 0x08, // MNT_NOSUID
			subtype: None,
		}
	}

	pub fn set_default_permissions(
		&mut self,
		default_permissions: bool,
	) -> &mut Self {
		self.default_permissions = default_permissions;
		self
	}

	pub fn set_mount_flags(&mut self, flags: i32) -> &mut Self {
		self.mount_flags = flags;
		self
	}

	pub fn set_mount_subtype(&mut self, subtype: &'a [u8]) -> &mut Self {
		self.subtype = Some(subtype);
		self
	}

	pub fn mount(
		&self,
		target_path: &[u8], // nul-terminated string
	) -> Result<LibcStream, MountError<LibcError>> {
		if !valid_cstr(target_path) {
			return Err(MountError::InvalidTargetPath);
		}
		if let Some(subtype) = self.subtype {
			if !valid_cstr(subtype) {
				return Err(MountError::InvalidMountSubtype);
			}
		}

		let file = LibcStream::dev_fuse().map_err(|e| MountError::Other(e))?;

		let mut fd_buf = [0u8; 32];
		file.fmt_raw_fd(&mut fd_buf);
		let mut iovecs = [
			// fstype
			IoVec::global(b"fstype\0"),
			IoVec::global(b"fusefs\0"),

			// from
			IoVec::global(b"from\0"),
			IoVec::global(b"/dev/fuse\0"),

			// fspath
			IoVec::global(b"fspath\0"),
			IoVec::borrow(target_path),

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

		if self.default_permissions {
			iovecs[iovecs_len] = IoVec::global(b"default_permissions\0");
			iovecs[iovecs_len + 1] = IoVec::global(b"\0");
			iovecs_len += 2;
		}

		if let Some(subtype) = self.subtype {
			iovecs[iovecs_len] = IoVec::global(b"subtype=\0");
			iovecs[iovecs_len + 1] = IoVec::borrow(subtype);
			iovecs_len += 2;
		}

		let nmount_rc = unsafe {
			libc::nmount(
				iovecs.as_mut_ptr() as *mut libc::iovec,
				iovecs_len as libc::c_uint,
				self.mount_flags,
			)
		};
		if nmount_rc == -1 {
			return Err(MountError::Other(LibcError::last_os_error()));
		}

		Ok(file)
	}
}

fn valid_cstr(buf: &[u8]) -> bool {
	let len = buf.len();
	if len == 0 {
		return false;
	}
	if buf[len - 1] != 0 {
		return false;
	}
	if buf[..len - 1].contains(&0) {
		return false;
	}
	true
}
