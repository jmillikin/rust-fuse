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
use std::io;

pub(crate) fn getuid() -> u32 {
	unsafe { target::getuid() as u32 }
}

pub(crate) fn getgid() -> u32 {
	unsafe { target::getgid() as u32 }
}

pub(crate) fn mount(
	source: &CStr,
	target: &CStr,
	fstype: &CStr,
	mountflags: u32,
	data: &[u8],
) -> io::Result<()> {
	let rc = unsafe { target::mount(source, target, fstype, mountflags, data) };
	if rc == 0 {
		return Ok(());
	}
	Err(io::Error::from_raw_os_error(-(rc as isize) as i32))
}

#[cfg(target_arch = "arm")] // EABI
mod target {
	#![allow(non_upper_case_globals)]

	use std::ffi::CStr;

	const SYS_getuid32: usize = 199;
	const SYS_getgid32: usize = 200;
	const SYS_mount: usize = 21;

	pub(super) unsafe fn getuid() -> usize {
		let rc: usize;
		core::arch::asm!(
			"swi #0",
			in("r7") SYS_getuid32,
			lateout("r0") rc,
		);
		rc
	}

	pub(super) unsafe fn getgid() -> usize {
		let rc: usize;
		core::arch::asm!(
			"swi #0",
			in("r7") SYS_getgid32,
			lateout("r0") rc,
		);
		rc
	}

	pub(super) unsafe fn mount(
		source: &CStr,
		target: &CStr,
		fstype: &CStr,
		mountflags: u32,
		data: &[u8],
	) -> usize {
		let mut rc: usize;
		core::arch::asm!(
			"swi #0",
			in("r7") SYS_mount,
			in("r0") source.as_ptr(),
			in("r1") target.as_ptr(),
			in("r2") fstype.as_ptr(),
			in("r3") mountflags,
			in("r4") data.as_ptr(),
			lateout("r0") rc,
		);
		rc
	}
}

#[cfg(target_arch = "x86")]
mod target {
	#![allow(non_upper_case_globals)]

	use std::ffi::CStr;

	const SYS_getuid32: usize = 199;
	const SYS_getgid32: usize = 200;
	const SYS_mount: usize = 21;

	pub(super) unsafe fn getuid() -> usize {
		let rc: usize;
		core::arch::asm!(
			"int 0x80",
			in("eax") SYS_getuid32,
			lateout("eax") rc,
		);
		rc
	}

	pub(super) unsafe fn getgid() -> usize {
		let rc: usize;
		core::arch::asm!(
			"int 0x80",
			in("eax") SYS_getgid32,
			lateout("eax") rc,
		);
		rc
	}

	pub(super) unsafe fn mount(
		source: &CStr,
		target: &CStr,
		fstype: &CStr,
		mountflags: u32,
		data: &[u8],
	) -> usize {
		let mut rc: usize;
		core::arch::asm!(
			"int 0x80",
			in("eax") SYS_mount,
			in("ebx") source.as_ptr(),
			in("ecx") target.as_ptr(),
			in("edx") fstype.as_ptr(),
			in("esi") mountflags,
			in("edi") data.as_ptr(),
			lateout("eax") rc,
		);
		rc
	}
}

#[cfg(target_arch = "x86_64")]
mod target {
	#![allow(non_upper_case_globals)]

	use std::ffi::CStr;

	const SYS_getuid: usize = 102;
	const SYS_getgid: usize = 104;
	const SYS_mount: usize = 165;

	pub(super) unsafe fn getuid() -> usize {
		let rc: usize;
		core::arch::asm!(
			"syscall",
			in("rax") SYS_getuid,
			out("rcx") _,
			out("r11") _,
			lateout("rax") rc,
		);
		rc
	}

	pub(super) unsafe fn getgid() -> usize {
		let rc: usize;
		core::arch::asm!(
			"syscall",
			in("rax") SYS_getgid,
			out("rcx") _,
			out("r11") _,
			lateout("rax") rc,
		);
		rc
	}

	pub(super) unsafe fn mount(
		source: &CStr,
		target: &CStr,
		fstype: &CStr,
		mountflags: u32,
		data: &[u8],
	) -> usize {
		let mut rc: usize;
		core::arch::asm!(
			"syscall",
			in("rax") SYS_mount,
			in("rdi") source.as_ptr(),
			in("rsi") target.as_ptr(),
			in("rdx") fstype.as_ptr(),
			in("r10") mountflags,
			in("r8") data.as_ptr(),
			out("rcx") _,
			out("r11") _,
			lateout("rax") rc,
		);
		rc
	}
}
