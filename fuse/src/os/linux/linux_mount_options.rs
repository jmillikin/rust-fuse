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

use std::ffi::CStr;
use std::ffi::{CString, OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::{fs, io};

use libc::c_ulong;
use libc::c_void;

/// **\[UNSTABLE\]**
pub struct FuseMountOptions {
	device_path: PathBuf,
	mount_source: String,
	mount_subtype: String,
	user_id: Option<u32>,
	group_id: Option<u32>,
	root_mode: Option<u32>,
}

impl crate::FuseMountOptions for FuseMountOptions {
	type Mount = FuseMount;
}

impl FuseMountOptions {
	pub fn new() -> Self {
		Self {
			device_path: PathBuf::from("/dev/fuse"),
			mount_source: "".to_string(),
			mount_subtype: "".to_string(),
			user_id: None,
			group_id: None,
			root_mode: None,
		}
	}

	pub fn set_mount_source(mut self, mount_source: &str) -> Self {
		self.mount_source = mount_source.to_string();
		self
	}

	pub fn set_mount_subtype(mut self, mount_subtype: &str) -> Self {
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

	fn mount_flags(&self) -> c_ulong {
		// int flags = MS_NOSUID | MS_NODEV;

		// unimplemented!()
		return 6;
	}

	fn mount_source_cstr(&self) -> io::Result<CString> {
		let name = &self.mount_source;
		if name == "" {
			return Ok(CString::new("fuse").unwrap());
		}
		cstr_from_osstr(OsStr::new(&name))
	}

	fn mount_type_cstr(&self) -> io::Result<CString> {
		let subtype = &self.mount_subtype;
		if subtype == "" {
			return Ok(CString::new("fuse").unwrap());
		}

		let mut buf = OsString::from("fuse.");
		buf.push(subtype);
		cstr_from_osstr(&buf)
	}

	fn mount_data(&self, fd: RawFd, root_mode: u32) -> io::Result<CString> {
		let user_id = self.user_id.unwrap_or_else(|| unsafe { libc::getuid() });
		let group_id =
			self.group_id.unwrap_or_else(|| unsafe { libc::getgid() });

		let mut out = Vec::new();
		out.push(format!("fd={},rootmode={:o}", fd, root_mode));
		out.push(format!("user_id={}", user_id));
		out.push(format!("group_id={}", group_id));
		let joined = CString::new(out.join(",")).unwrap();

		/*
		if joined.as_bytes_with_nul().len() > libc::getpagesize()? {
			return Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"mount options too long",
			));
		}
		*/

		Ok(joined)
	}
}

/// **\[UNSTABLE\]**
pub struct FuseMount {
	mount_target: PathBuf,
}

impl crate::FuseMount for FuseMount {
	type Options = FuseMountOptions;

	#[doc(hidden)]
	fn mount(
		mount_target: &std::path::Path,
		options: Option<Self::Options>,
	) -> io::Result<(std::fs::File, Self)> {
		let options = match options {
			Some(x) => x,
			None => FuseMountOptions::new(),
		};

		let mount_target_cstr = cstr_from_osstr(mount_target.as_os_str())?;
		let mount_source_cstr = options.mount_source_cstr()?;
		let mount_type_cstr = options.mount_type_cstr()?;
		let mount_flags = options.mount_flags();

		let root_mode = match options.root_mode {
			Some(mode) => mode,
			None => {
				let meta = fs::metadata(mount_target)?;
				meta.mode()
			},
		};

		let file = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open(&options.device_path)?;
		let fd = file.as_raw_fd();

		let mount_data = options.mount_data(fd, root_mode)?;
		libc_mount(
			&mount_source_cstr,
			&mount_target_cstr,
			&mount_type_cstr,
			mount_flags,
			mount_data.as_ptr() as *const c_void,
		)?;

		Ok((
			file,
			Self {
				mount_target: mount_target.to_path_buf(),
			},
		))
	}

	#[doc(hidden)]
	fn unmount(self) -> io::Result<()> {
		println!("Linux unmount not implemented yet");
		let _ = self.mount_target;
		Ok(())
	}
}

fn cstr_from_osstr(x: &OsStr) -> io::Result<CString> {
	match CString::new(x.as_bytes()) {
		Ok(val) => Ok(val),
		Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
	}
}

fn libc_mount(
	source: &CStr,
	target: &CStr,
	filesystemtype: &CStr,
	mountflags: c_ulong,
	data: *const c_void,
) -> io::Result<()> {
	let rc = unsafe {
		libc::mount(
			source.as_ptr(),
			target.as_ptr(),
			filesystemtype.as_ptr(),
			mountflags,
			data,
		)
	};
	if rc == -1 {
		return Err(io::Error::last_os_error());
	}
	assert!(rc == 0);
	Ok(())
}
