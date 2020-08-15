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
use std::path::PathBuf;
use std::{fs, io};

use crate::channel::{self, Channel, FileChannel};

#[path = "linux_syscalls.rs"]
mod syscalls;

pub struct FuseChannelBuilder {
	device_path: PathBuf,
	mount_source: String,
	mount_subtype: String,
	user_id: Option<u32>,
	group_id: Option<u32>,
	root_mode: Option<u32>,
}

impl FuseChannelBuilder {
	pub fn new() -> FuseChannelBuilder {
		Self {
			device_path: PathBuf::from("/dev/fuse"),
			mount_source: "".to_string(),
			mount_subtype: "".to_string(),
			user_id: None,
			group_id: None,
			root_mode: None,
		}
	}

	pub fn set_mount_source(&mut self, mount_source: &str) -> &mut Self {
		self.mount_source = mount_source.to_string();
		self
	}

	pub fn set_mount_subtype(&mut self, mount_subtype: &str) -> &mut Self {
		self.mount_subtype = mount_subtype.to_string();
		self
	}

	pub fn set_user_id(&mut self, uid: u32) -> &mut Self {
		self.user_id = Some(uid);
		self
	}

	pub fn set_group_id(&mut self, gid: u32) -> &mut Self {
		self.group_id = Some(gid);
		self
	}

	pub fn set_root_mode(&mut self, mode: u32) -> &mut Self {
		self.root_mode = Some(mode);
		self
	}

	pub fn mount<Path>(&mut self, mount_target: Path) -> io::Result<FuseChannel>
	where
		Path: AsRef<std::path::Path>,
	{
		let mount_target = mount_target.as_ref();

		let mount_target_cstr = cstr_from_osstr(mount_target.as_os_str())?;
		let mount_source_cstr = self.mount_source_cstr()?;
		let mount_type_cstr = self.mount_type_cstr()?;
		let mount_flags = self.mount_flags();

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
			.open(&self.device_path)?;
		let fd = file.as_raw_fd();

		let mount_data = self.mount_data(fd, root_mode)?;
		syscalls::mount(
			&mount_source_cstr,
			&mount_target_cstr,
			&mount_type_cstr,
			mount_flags,
			mount_data.to_bytes_with_nul(),
		)?;

		Ok(FuseChannel(FileChannel::new(file)))
	}

	fn mount_flags(&self) -> u32 {
		// int flags = MS_NOSUID | MS_NODEV;

		// unimplemented!()
		return 6;
	}

	fn mount_source_cstr(&self) -> Result<CString, io::Error> {
		let name = &self.mount_source;
		if name == "" {
			return Ok(CString::new("fuse").unwrap());
		}
		cstr_from_osstr(OsStr::new(&name))
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

		Ok(joined)
	}
}

pub struct FuseChannel(FileChannel);

impl channel::FuseChannel for FuseChannel {}

impl Channel for FuseChannel {
	type Error = io::Error;

	fn send(&self, buf: &[u8]) -> Result<(), io::Error> {
		self.0.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), io::Error> {
		self.0.send_vectored(bufs)
	}

	fn receive(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
		self.0.receive(buf)
	}

	fn try_clone(&self) -> Result<Self, io::Error> {
		Ok(FuseChannel(self.0.try_clone()?))
	}
}

fn cstr_from_osstr(x: &OsStr) -> Result<CString, io::Error> {
	match CString::new(x.as_bytes()) {
		Ok(val) => Ok(val),
		Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
	}
}
