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

use core::num::NonZeroU16;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::{env, ffi, panic, path, sync, thread};

struct PrintHooks {}

impl fuse::ServerHooks for PrintHooks {
	fn unknown_request(&self, request: &fuse::UnknownRequest) {
		println!("\n[unknown_request]\n{:#?}", request);
	}

	fn unhandled_request(&self, request_header: &fuse::server::RequestHeader) {
		println!("\n[unhandled_request]\n{:#?}", request_header);
	}

	fn request_error(
		&self,
		request_header: &fuse::server::RequestHeader,
		err: fuse::Error,
	) {
		println!("\n[request_error]\n{:#?}", request_header);
		println!("{:#?}", err);
	}

	fn response_error(
		&self,
		request_header: &fuse::server::RequestHeader,
		code: Option<NonZeroU16>,
	) {
		println!("\n[response_error]\n{:#?}", request_header);
		println!("{:#?}", code);
	}

	fn async_channel_error(
		&self,
		request_header: &fuse::server::RequestHeader,
		code: Option<NonZeroU16>,
	) {
		println!("\n[async_channel_error]\n{:#?}", request_header);
		println!("{:#?}", code);
	}
}

pub fn interop_test(
	fs: impl fuse::FuseHandlers + Send + 'static,
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) {
	let mut mkdtemp_template = {
		let mut tmp = env::temp_dir();
		tmp.push("rust_fuse.XXXXXX\x00");
		tmp.into_os_string().into_vec()
	};

	{
		let template_ptr = mkdtemp_template.as_mut_ptr() as *mut libc::c_char;
		let mkdtemp_ret = unsafe { libc::mkdtemp(template_ptr) };
		assert!(!mkdtemp_ret.is_null());
	}
	mkdtemp_template.truncate(mkdtemp_template.len() - 1);
	let mount_cstr = ffi::CString::new(mkdtemp_template.clone()).unwrap();
	let mount_path = path::Path::new(ffi::OsStr::from_bytes(&mkdtemp_template))
		.to_path_buf();

	let server_ready = sync::Arc::new(sync::Barrier::new(2));
	let server_thread = {
		let ready = sync::Arc::clone(&server_ready);
		let mount_path = mount_path.clone();
		thread::spawn(move || {
			use fuse::os::linux;
			let mut srv = linux::FuseServerBuilder::new(mount_path, fs)
				.set_mount(
					linux::SyscallFuseMount::new()
						.set_mount_source("ruse_fuse_test")
						.set_mount_subtype("ruse_fuse_test"),
				)
				.set_hooks(PrintHooks {})
				.build()?;
			ready.wait();
			srv.executor_mut().run()
		})
	};

	server_ready.wait();
	let test_result = panic::catch_unwind(|| test_fn(&mount_path));

	let umount_rc = unsafe { libc::umount(mount_cstr.as_ptr()) };
	if umount_rc == -1 {
		unsafe {
			libc::umount2(mount_cstr.as_ptr(), libc::MNT_FORCE)
		};
	}
	let server_result = server_thread.join();

	if let Err(err) = test_result {
		panic::resume_unwind(err);
	} else {
		match server_result {
			Err(err) => panic::resume_unwind(err),
			Ok(_) => {
				//fuse_result.unwrap();
				assert_eq!(umount_rc, 0);
			},
		}
	}
}

pub fn path_cstr(path: std::path::PathBuf) -> ffi::CString {
	ffi::CString::new(path.as_os_str().as_bytes()).unwrap()
}

pub fn diff_str(want: &str, got: &str) -> Option<String> {
	let mut out = String::new();
	let mut ok = true;
	for result in diff::lines(want, got) {
		match result {
			diff::Result::Left(l) => {
				ok = false;
				out.push_str("- ");
				out.push_str(l);
				out.push('\n');
			},
			diff::Result::Both(l, _) => {
				out.push_str("  ");
				out.push_str(l);
				out.push('\n');
			},
			diff::Result::Right(r) => {
				ok = false;
				out.push_str("+ ");
				out.push_str(r);
				out.push('\n');
			},
		}
	}

	if ok {
		return None;
	}
	Some(out)
}

pub fn errno() -> libc::c_int {
	unsafe {
		#[cfg(target_os = "linux")]
		return *libc::__errno_location();

		#[cfg(target_os = "freebsd")]
		return *libc::__error();
	}
}
