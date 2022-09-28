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

// use core::ffi::CStr;
use std::ffi::CStr;

use core::marker::PhantomData;

use linux_errno::Error;
use linux_syscall::{self as syscall, syscall, Result as _, ResultSize as _};

pub(crate) const AT_FDCWD: i32 = -100;

pub(crate) unsafe fn close(fd: i32) -> Result<(), Error> {
	syscall!(syscall::SYS_close, fd).check()
}

pub(crate) type OpenFlag = u32;

pub(crate) const O_RDWR: OpenFlag = 1 << 1;

#[cfg(not(any(
	target_arch = "alpha",
	target_arch = "parisc",
	target_arch = "sparc",
)))]
pub(crate) const O_CLOEXEC: OpenFlag = 1 << 19;

#[repr(C)]
struct open_how {
	flags:   u64,
	mode:    u64,
	resolve: u64,
}

pub(crate) unsafe fn open(
	dir_fd: i32,
	filename: &CStr,
	flags: OpenFlag,
	mode: u32,
) -> Result<i32, Error> {
	let openat2_opts = &open_how {
		flags: u64::from(flags),
		mode: u64::from(mode),
		resolve: 0,
	};
	match syscall!(
		syscall::SYS_openat2,
		dir_fd,
		filename.as_ptr(),
		openat2_opts as *const open_how,
		core::mem::size_of::<open_how>(),
	).try_isize() {
		Ok(fd) => return Ok(fd as i32),
		Err(linux_errno::ENOSYS) => {},
		Err(err) => return Err(err),
	};
	syscall!(
		syscall::SYS_openat,
		dir_fd,
		filename.as_ptr(),
		flags,
		mode,
	).try_isize().map(|fd| fd as i32)
}

pub(crate) unsafe fn read(fd: i32, buf: &mut [u8]) -> Result<usize, Error> {
	let rc = syscall!(syscall::SYS_read, fd, buf.as_mut_ptr(), buf.len());
	rc.try_usize()
}

#[repr(C)]
pub(crate) struct IoVec<'a> {
	iov_base: *const core::ffi::c_void,
	iov_len:  usize,
	_phantom: PhantomData<&'a [u8]>,
}

impl<'a> IoVec<'a> {
	pub(crate) fn borrow(buf: &'a [u8]) -> Self {
		IoVec {
			iov_base: buf.as_ptr() as *const core::ffi::c_void,
			iov_len: buf.len(),
			_phantom: PhantomData,
		}
	}
}

pub(crate) unsafe fn writev(fd: i32, iov: &[IoVec]) -> Result<usize, Error> {
	let rc = syscall!(syscall::SYS_writev, fd, iov.as_ptr(), iov.len());
	rc.try_usize()
}

pub(crate) fn getuid() -> u32 {
	#[allow(unused_mut)]
	let mut sys_getuid = linux_syscall::SYS_getuid;

	#[cfg(any(
		target_arch = "arm",
		target_arch = "x86",
	))]
	{
		sys_getuid = linux_syscall::SYS_getuid32;
	}

	let rc = unsafe { syscall!(sys_getuid) };
	rc.as_usize_unchecked() as u32
}

pub(crate) fn getgid() -> u32 {
	#[allow(unused_mut)]
	let mut sys_getgid = linux_syscall::SYS_getgid;

	#[cfg(any(
		target_arch = "arm",
		target_arch = "x86",
	))]
	{
		sys_getgid = linux_syscall::SYS_getgid32;
	}

	let rc = unsafe { syscall!(sys_getgid) };
	rc.as_usize_unchecked() as u32
}

pub(crate) unsafe fn mount(
	source: &CStr,
	target: &CStr,
	fstype: &CStr,
	mountflags: u32,
	data: &[u8],
) -> Result<(), Error> {
	syscall!(
		linux_syscall::SYS_mount,
		source.as_ptr(),
		target.as_ptr(),
		fstype.as_ptr(),
		mountflags,
		data.as_ptr(),
	).check()
}

#[repr(C)]
pub(crate) struct kernel_statx {
	stx_mask: u32,
	stx_blksize: u32,
	stx_attributes: u64,
	stx_nlink: u32,
	stx_uid: u32,
	stx_gid: u32,
	pub(crate) stx_mode: u16,
	_pad1: [u16; 1],
	stx_ino: u64,
	stx_size: u64,
	stx_blocks: u64,
	stx_attributes_mask: u64,
	stx_atime: kernel_statx_timestamp,
	stx_btime: kernel_statx_timestamp,
	stx_ctime: kernel_statx_timestamp,
	stx_mtime: kernel_statx_timestamp,
	stx_rdev_major: u32,
	stx_rdev_minor: u32,
	stx_dev_major: u32,
	stx_dev_minor: u32,
	_pad2: [u64; 14],
}

#[repr(C)]
pub(crate) struct kernel_statx_timestamp {
	tv_sec: i64,
	tv_nsec: u32,
	_pad: i32,
}

pub(crate) const STATX_MODE: u32 = 1 << 1;

pub(crate) unsafe fn statx(
	dir_fd: i32,
	filename: &CStr,
	flags: u32,
	mask: u32,
) -> Result<kernel_statx, Error> {
	let mut statx: kernel_statx = core::mem::zeroed();
	syscall!(
		linux_syscall::SYS_statx,
		dir_fd,
		filename.as_ptr(),
		flags,
		mask,
		&mut statx as *mut kernel_statx,
	).check()?;
	Ok(statx)
}
