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

use std::ffi::{CString, OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::RawFd;
use std::{fs, io, path};

use fuse::os::linux::{MountData, MountOptions};

use crate::io::stream::LibcStream;

const MS_NOSUID: u32 = 0x2;
const MS_NODEV: u32 = 0x4;

// This is technically incorrect, because Linux can be compiled with
// different page sizes (and often is on e.g. ARM). But we're using this value
// only as a maximum length limit for `mount(2)` data, so hardcoding should
// be fine.
const PAGE_SIZE: usize = 4096;

struct FuseMountOptions {
	mount_source: OsString,
	mount_subtype: OsString,
	mount_flags: u32,
	user_id: Option<u32>,
	group_id: Option<u32>,
	root_mode: Option<u32>,
}

impl FuseMountOptions {
	fn new() -> FuseMountOptions {
		Self {
			mount_source: OsString::new(),
			mount_subtype: OsString::new(),
			mount_flags: MS_NOSUID | MS_NODEV,
			user_id: None,
			group_id: None,
			root_mode: None,
		}
	}

	fn mount_source_cstr(&self) -> Result<CString, io::Error> {
		let name = &self.mount_source;
		if name == "" {
			return Ok(CString::new("fuse").unwrap());
		}
		cstr_from_osstr(&name)
	}

	fn mount_type_cstr(&self) -> Result<CString, io::Error> {
		Ok(CString::new("fuse").unwrap())
	}

	fn mount_data(
		&self,
		fd: RawFd,
		root_mode: u32,
		user_id: u32,
		group_id: u32,
	) -> Result<CString, io::Error> {
		let mut opts = MountOptions::new();
		opts.set_fuse_device_fd(Some(fd as u32));
		opts.set_root_mode(Some(root_mode));
		opts.set_user_id(Some(user_id));
		opts.set_group_id(Some(group_id));

		let fs_subtype: CString;
		if !self.mount_subtype.is_empty() {
			fs_subtype = cstr_from_osstr(&self.mount_subtype)?;
			opts.set_fs_subtype(Some(&fs_subtype));
		}

		let mut buf = [0u8; PAGE_SIZE];
		match MountData::new(&mut buf, &opts) {
			Some(mount_data) => Ok(CString::from(mount_data.as_cstr())),
			None => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"mount data length > PAGE_SIZE",
			)),
		}
	}
}

pub struct LibcFuseMount(FuseMountOptions);

impl LibcFuseMount {
	pub fn new() -> LibcFuseMount {
		Self(FuseMountOptions::new())
	}

	pub fn set_mount_source(mut self, mount_source: impl AsRef<OsStr>) -> Self {
		self.0.mount_source = mount_source.as_ref().to_os_string();
		self
	}

	pub fn set_mount_subtype(
		mut self,
		mount_subtype: impl AsRef<OsStr>,
	) -> Self {
		self.0.mount_subtype = mount_subtype.as_ref().to_os_string();
		self
	}

	pub fn set_mount_flags(mut self, mount_flags: u32) -> Self {
		self.0.mount_flags = mount_flags;
		self
	}

	pub fn set_user_id(mut self, uid: u32) -> Self {
		self.0.user_id = Some(uid);
		self
	}

	pub fn set_group_id(mut self, gid: u32) -> Self {
		self.0.group_id = Some(gid);
		self
	}

	pub fn set_root_mode(mut self, mode: u32) -> Self {
		self.0.root_mode = Some(mode);
		self
	}

	pub fn mount(
		self,
		mount_target: &path::Path,
	) -> Result<LibcStream, io::Error> {
		let mount_target_cstr = cstr_from_osstr(mount_target.as_os_str())?;
		let mount_source_cstr = self.0.mount_source_cstr()?;
		let mount_type_cstr = self.0.mount_type_cstr()?;

		let root_mode = match self.0.root_mode {
			Some(mode) => mode,
			None => {
				let meta = fs::metadata(mount_target)?;
				meta.mode()
			},
		};

		let stream = LibcStream::dev_fuse()?;
		let fd = stream.as_raw_fd();

		let user_id =
			self.0.user_id.unwrap_or_else(|| unsafe { libc::getuid() });
		let group_id =
			self.0.group_id.unwrap_or_else(|| unsafe { libc::getgid() });

		let mount_data = self.0.mount_data(fd, root_mode, user_id, group_id)?;
		unsafe {
			let rc = libc::mount(
				mount_source_cstr.as_ptr(),
				mount_target_cstr.as_ptr(),
				mount_type_cstr.as_ptr(),
				self.0.mount_flags as libc::c_ulong,
				mount_data.to_bytes_with_nul().as_ptr() as *const libc::c_void,
			);
			if rc != 0 {
				return Err(std::io::Error::last_os_error());
			}
		};
		Ok(stream)
	}
}

fn cstr_from_osstr(x: &OsStr) -> Result<CString, io::Error> {
	match CString::new(x.as_bytes()) {
		Ok(val) => Ok(val),
		Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
	}
}
