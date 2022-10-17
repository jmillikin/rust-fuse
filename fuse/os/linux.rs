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

use core::ffi;
use core::fmt;
use core::fmt::Write;

const CSTR_FUSE: &ffi::CStr = unsafe {
	ffi::CStr::from_bytes_with_nul_unchecked(b"fuse\0")
};

// MountOptions {{{

#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	allow_other: bool,
	block_size: Option<u32>,
	default_permissions: bool,
	fs_subtype: Option<&'a ffi::CStr>,
	fs_type: Option<&'a ffi::CStr>,
	fuse_device_fd: Option<u32>,
	group_id: Option<u32>,
	max_read: Option<u32>,
	root_mode: Option<u32>,
	source: Option<&'a ffi::CStr>,
	user_id: Option<u32>,
	_no_std: core::marker::PhantomData<&'a ()>,
}

impl<'a> MountOptions<'a> {
	#[must_use]
	pub fn new() -> Self {
		MountOptions {
			allow_other: false,
			block_size: None,
			default_permissions: false,
			fs_subtype: None,
			fs_type: None,
			fuse_device_fd: None,
			group_id: None,
			max_read: None,
			root_mode: None,
			source: None,
			user_id: None,
			_no_std: core::marker::PhantomData,
		}
	}

	#[must_use]
	pub fn allow_other(&self) -> bool {
		self.allow_other
	}

	pub fn set_allow_other(&mut self, allow_other: bool) {
		self.allow_other = allow_other;
	}

	#[must_use]
	pub fn block_size(&self) -> Option<u32> {
		self.block_size
	}

	pub fn set_block_size(&mut self, block_size: Option<u32>) {
		self.block_size = block_size;
	}

	#[must_use]
	pub fn default_permissions(&self) -> bool {
		self.default_permissions
	}

	pub fn set_default_permissions(&mut self, default_permissions: bool) {
		self.default_permissions = default_permissions;
	}

	#[must_use]
	pub fn fs_type(&self) -> &'a ffi::CStr {
		self.fs_type.unwrap_or(CSTR_FUSE)
	}

	pub fn set_fs_type(&mut self, fs_type: Option<&'a ffi::CStr>) {
		self.fs_type = fs_type;
	}

	#[must_use]
	pub fn fs_subtype(&self) -> Option<&'a ffi::CStr> {
		self.fs_subtype
	}

	pub fn set_fs_subtype(&mut self, fs_subtype: Option<&'a ffi::CStr>) {
		self.fs_subtype = fs_subtype;
	}

	#[must_use]
	pub fn fuse_device_fd(&self) -> Option<u32> {
		self.fuse_device_fd
	}

	pub fn set_fuse_device_fd(&mut self, fuse_device_fd: Option<u32>) {
		self.fuse_device_fd = fuse_device_fd;
	}

	#[must_use]
	pub fn group_id(&self) -> Option<u32> {
		self.group_id
	}

	pub fn set_group_id(&mut self, group_id: Option<u32>) {
		self.group_id = group_id;
	}

	#[must_use]
	pub fn max_read(&self) -> Option<u32> {
		self.max_read
	}

	pub fn set_max_read(&mut self, max_read: Option<u32>) {
		self.max_read = max_read;
	}

	#[must_use]
	pub fn root_mode(&self) -> Option<u32> {
		self.root_mode
	}

	pub fn set_root_mode(&mut self, root_mode: Option<u32>) {
		self.root_mode = root_mode;
	}

	#[must_use]
	pub fn source(&self) -> &'a ffi::CStr {
		self.source.unwrap_or(CSTR_FUSE)
	}

	pub fn set_source(&mut self, source: Option<&'a ffi::CStr>) {
		self.source = source;
	}

	#[must_use]
	pub fn user_id(&self) -> Option<u32> {
		self.user_id
	}

	pub fn set_user_id(&mut self, user_id: Option<u32>) {
		self.user_id = user_id;
	}
}

// }}}

// MountData {{{

#[derive(Copy, Clone)]
pub struct MountData<'a> {
	buf: &'a ffi::CStr,
}

impl<'a> MountData<'a> {
	#[must_use]
	pub fn new(buf: &'a mut [u8], options: &MountOptions) -> Option<Self> {
		let mut w = BufWriter { buf, count: 0 };
		if write_mount_data(&mut w, options).is_err() {
			return None;
		}
		let count = w.count;

		let buf_cstr = unsafe {
			ffi::CStr::from_bytes_with_nul_unchecked(&buf[..count])
		};
		Some(Self { buf: buf_cstr })
	}

	#[must_use]
	pub fn as_cstr(&self) -> &'a ffi::CStr {
		self.buf
	}
}

fn write_mount_data(w: &mut BufWriter, opts: &MountOptions) -> fmt::Result {
	let comma = ",";
	let mut sep = "";

	// Output fd= first so it's easy to locate in debug logs and strace output.
	if let Some(fuse_device_fd) = opts.fuse_device_fd {
		write!(w, "fd={}", fuse_device_fd)?;
		sep = comma;
	}

	// Other options are written in order by key.
	if opts.allow_other {
		write!(w, "{}allow_other", sep)?;
		sep = comma;
	}
	if let Some(block_size) = opts.block_size {
		write!(w, "{}blksize={}", sep, block_size)?;
		sep = comma;
	}
	if opts.default_permissions {
		write!(w, "{}default_permissions", sep)?;
		sep = comma;
	}
	if let Some(group_id) = opts.group_id {
		write!(w, "{}group_id={}", sep, group_id)?;
		sep = comma;
	}
	if let Some(max_read) = opts.max_read {
		write!(w, "{}max_read={}", sep, max_read)?;
		sep = comma;
	}
	if let Some(root_mode) = opts.root_mode {
		write!(w, "{}rootmode={:o}", sep, root_mode)?;
		sep = comma;
	}
	if let Some(fs_subtype) = opts.fs_subtype {
		if !cstr_is_empty(fs_subtype) {
			write!(w, "{}subtype=", sep)?;
			w.write_cstr(fs_subtype)?;
			sep = comma;
		}
	}
	if let Some(user_id) = opts.user_id {
		write!(w, "{}user_id={}", sep, user_id)?;
	}

	// Ensure the output is terminated by NUL. Although the `mount()` data
	// parameter is `void*`, FUSE expects its mount data to be a C string.
	w.write_bytes(&[0])?;
	Ok(())
}

// }}}

// BufWriter {{{

struct BufWriter<'a> {
	buf: &'a mut [u8],
	count: usize,
}

impl BufWriter<'_> {
	fn write_bytes(&mut self, b: &[u8]) -> fmt::Result {
		let avail = &mut self.buf[self.count..];
		if b.len() > avail.len() {
			return Err(fmt::Error);
		}
		avail[..b.len()].copy_from_slice(b);
		self.count += b.len();
		Ok(())
	}

	fn write_cstr(&mut self, s: &ffi::CStr) -> fmt::Result {
		let b = s.to_bytes();
		if b.contains(&b',') {
			return Err(fmt::Error);
		}
		self.write_bytes(b)
	}
}

impl fmt::Write for BufWriter<'_> {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_bytes(s.as_bytes())
	}
}

// }}}

// https://github.com/rust-lang/rust/issues/102444
fn cstr_is_empty(s: &ffi::CStr) -> bool {
	unsafe { s.as_ptr().read() == 0 }
}
