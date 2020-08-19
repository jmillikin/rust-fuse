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
use std::os::unix::io::{AsRawFd, RawFd};
use std::{fs, io, path};

use super::linux_syscalls as syscalls;

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub trait FuseMount {
	fn fuse_mount(self, mount_target: &path::Path) -> io::Result<fs::File>;
}

#[cfg_attr(doc, doc(cfg(not(feature = "no_std"))))]
pub struct SyscallFuseMount {
	dev_fuse: path::PathBuf,
	mount_source: OsString,
	mount_subtype: OsString,
	mount_flags: u32,
	user_id: Option<u32>,
	group_id: Option<u32>,
	root_mode: Option<u32>,
}

const MS_NOSUID: u32 = 0x2;
const MS_NODEV: u32 = 0x4;

// This is technically incorrect, because Linux can be compiled with
// different page sizes (and often is on e.g. ARM). But we're using this value
// only as a maximum length limit for `mount(2)` data, so hardcoding should
// be fine.
const PAGE_SIZE: usize = 4096;

impl SyscallFuseMount {
	pub fn new() -> SyscallFuseMount {
		Self {
			dev_fuse: path::PathBuf::from("/dev/fuse"),
			mount_source: OsString::new(),
			mount_subtype: OsString::new(),
			mount_flags: MS_NOSUID | MS_NODEV,
			user_id: None,
			group_id: None,
			root_mode: None,
		}
	}

	pub fn set_mount_source(mut self, mount_source: impl AsRef<OsStr>) -> Self {
		self.mount_source = mount_source.as_ref().to_os_string();
		self
	}

	pub fn set_mount_subtype(
		mut self,
		mount_subtype: impl AsRef<OsStr>,
	) -> Self {
		self.mount_subtype = mount_subtype.as_ref().to_os_string();
		self
	}

	pub fn set_mount_flags(mut self, mount_flags: u32) -> Self {
		self.mount_flags = mount_flags;
		self
	}

	pub fn set_user_id(mut self, uid: u32) -> Self {
		self.user_id = Some(uid);
		self
	}

	pub fn set_group_id(mut self, gid: u32) -> Self {
		self.group_id = Some(gid);
		self
	}

	pub fn set_root_mode(mut self, mode: u32) -> Self {
		self.root_mode = Some(mode);
		self
	}

	fn mount_source_cstr(&self) -> Result<CString, io::Error> {
		let name = &self.mount_source;
		if name == "" {
			return Ok(CString::new("fuse").unwrap());
		}
		cstr_from_osstr(&name)
	}

	fn mount_type_cstr(&self) -> Result<CString, io::Error> {
		let subtype = &self.mount_subtype;
		if subtype == "" {
			return Ok(CString::new("fuse").unwrap());
		}

		let mut buf = OsString::from("fuse.");
		buf.push(subtype);
		cstr_from_osstr(&buf)
	}

	fn mount_data(
		&self,
		fd: RawFd,
		root_mode: u32,
	) -> Result<CString, io::Error> {
		let user_id = self.user_id.unwrap_or_else(|| syscalls::getuid());
		let group_id = self.group_id.unwrap_or_else(|| syscalls::getgid());

		let mut out = Vec::new();
		out.push(format!("fd={},rootmode={:o}", fd, root_mode));
		out.push(format!("user_id={}", user_id));
		out.push(format!("group_id={}", group_id));
		let joined = CString::new(out.join(",")).unwrap();

		let data_length = joined.as_bytes_with_nul().len();
		if data_length > PAGE_SIZE {
			return Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!(
					"mount data length ({}) exceeds PAGE_SIZE ({})",
					data_length, PAGE_SIZE
				),
			));
		}

		Ok(joined)
	}
}

impl FuseMount for SyscallFuseMount {
	fn fuse_mount(self, mount_target: &path::Path) -> io::Result<fs::File> {
		let mount_target_cstr = cstr_from_osstr(mount_target.as_os_str())?;
		let mount_source_cstr = self.mount_source_cstr()?;
		let mount_type_cstr = self.mount_type_cstr()?;

		let root_mode = match self.root_mode {
			Some(mode) => mode,
			None => {
				let meta = fs::metadata(mount_target)?;
				meta.mode()
			},
		};

		let file = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open(&self.dev_fuse)?;
		let fd = file.as_raw_fd();

		let mount_data = self.mount_data(fd, root_mode)?;
		syscalls::mount(
			&mount_source_cstr,
			&mount_target_cstr,
			&mount_type_cstr,
			self.mount_flags,
			mount_data.to_bytes_with_nul(),
		)?;

		Ok(file)
	}
}

fn cstr_from_osstr(x: &OsStr) -> Result<CString, io::Error> {
	match CString::new(x.as_bytes()) {
		Ok(val) => Ok(val),
		Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
	}
}
