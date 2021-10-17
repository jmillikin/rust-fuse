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

use core::mem::{self, MaybeUninit};
use core::num::{NonZeroU16, NonZeroUsize};

use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::{env, ffi, fs, io, panic, path, sync, thread};

struct PrintHooks {}

impl fuse::server::basic::ServerHooks for PrintHooks {
	fn unknown_request(&self, request: &fuse::UnknownRequest) {
		println!("\n[unknown_request]\n{:#?}", request);
	}

	fn unhandled_request(&self, header: &fuse::server::RequestHeader) {
		println!("\n[unhandled_request]\n{:#?}", header);
	}

	fn request_error(
		&self,
		header: &fuse::server::RequestHeader,
		err: fuse::io::RequestError,
	) {
		println!("\n[request_error]\n{:#?}", header);
		println!("{:#?}", err);
	}
}

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

pub struct DevCuse {
	dev_cuse: fs::File,
	pipe_r: fs::File,
}

impl DevCuse {
	fn new() -> (Self, /* pipe_w */ fs::File) {
		use std::os::unix::io::FromRawFd;

		let mut pipe_fds = [(0 as libc::c_int); 2];
		let pipe_rc = unsafe { libc::pipe(pipe_fds.as_mut_ptr()) };
		assert_eq!(pipe_rc, 0);

		let dev_cuse = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open("/dev/cuse")
			.unwrap();

		let pipe_r = unsafe { fs::File::from_raw_fd(pipe_fds[0]) };
		let pipe_w = unsafe { fs::File::from_raw_fd(pipe_fds[1]) };

		(Self { dev_cuse, pipe_r }, pipe_w)
	}
}

impl fuse::io::OutputStream for DevCuse {
	type Error = io::Error;

	fn send(&self, buf: &[u8]) -> Result<(), io::Error> {
		use std::io::Write;

		let write_size = Write::write(&mut &self.dev_cuse, buf)?;
		if write_size < buf.len() {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), io::Error> {
		use std::io::Write;

		let mut bufs_len: usize = 0;
		let io_slices: &[io::IoSlice] = {
			let mut uninit_bufs: [MaybeUninit<io::IoSlice>; N] =
				unsafe { MaybeUninit::uninit().assume_init() };
			for ii in 0..N {
				bufs_len += bufs[ii].len();
				uninit_bufs[ii] = MaybeUninit::new(io::IoSlice::new(bufs[ii]));
			}
			unsafe { mem::transmute::<_, &[io::IoSlice; N]>(&uninit_bufs) }
		};

		let write_size = Write::write_vectored(&mut &self.dev_cuse, io_slices)?;
		if write_size < bufs_len {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}
}

impl fuse::io::InputStream for DevCuse {
	type Error = io::Error;

	fn recv(&self, buf: &mut [u8]) -> Result<Option<NonZeroUsize>, io::Error> {
		use std::io::Read;
		use std::os::unix::io::AsRawFd;
		use fuse::io::ChannelError;

		let mut poll_fds: [libc::pollfd; 2] = [
			libc::pollfd {
				fd: self.dev_cuse.as_raw_fd(),
				events: libc::POLLIN,
				revents: 0,
			},
			libc::pollfd {
				fd: self.pipe_r.as_raw_fd(),
				events: 0,
				revents: 0,
			},
		];

		loop {
			let poll_rc = unsafe { libc::poll(
				poll_fds.as_mut_ptr(),
				poll_fds.len() as libc::nfds_t,
				-1, // timeout
			) };
			if poll_rc == libc::EINTR {
				continue;
			}
			assert!(poll_rc > 0);

			if (poll_fds[1].revents & libc::POLLERR) > 0 ||
			   (poll_fds[1].revents & libc::POLLHUP) > 0 {
				return Ok(None);
			}

			if (poll_fds[0].revents & libc::POLLIN) == 0 {
				continue;
			}

			match Read::read(&mut &self.dev_cuse, buf) {
				Ok(size) => return Ok(NonZeroUsize::new(size)),
				Err(err) => match err.error_code() {
					Some(fuse::ErrorCode::ENOENT) => {
						// The next request in the kernel buffer was interrupted before
						// it could be deleted. Try again.
					},
					Some(fuse::ErrorCode::EINTR) => {
						// Interrupted by signal. Try again.
					},
					_ => return Err(err),
				},
			}
		}
	}
}

extern "C" {
	#[link_name = "mktemp"]
	fn libc_mktemp(template: *mut libc::c_char) -> *mut libc::c_char;
}

const CUSE_DEV_MAJOR: libc::c_uint = 240; // "LOCAL/EXPERIMENTAL USE"
const CUSE_DEV_MINOR: libc::c_uint = 1;

pub fn cuse_interop_test(
	handlers: impl fuse::server::basic::CuseHandlers<DevCuse> + Send + 'static,
	test_fn: impl FnOnce(&path::Path) + panic::UnwindSafe,
) {
	let mut mktemp_template = {
		let mut tmp = path::PathBuf::from("/dev/");
		tmp.push("rust-cuse.XXXXXX\x00");
		tmp.into_os_string().into_vec()
	};

	{
		let template_ptr = mktemp_template.as_mut_ptr() as *mut libc::c_char;
		let mktemp_ret = unsafe { libc_mktemp(template_ptr) };
		assert!(!mktemp_ret.is_null());
	}
	mktemp_template.truncate(mktemp_template.len() - 1);
	let device_path_cstr = ffi::CString::new(mktemp_template.clone()).unwrap();
	let device_path = path::Path::new(ffi::OsStr::from_bytes(&mktemp_template))
		.to_path_buf();

	let mknod_rc = unsafe {
		let dev_t = libc::makedev(CUSE_DEV_MAJOR, CUSE_DEV_MINOR);
		libc::mknod(device_path_cstr.as_ptr(), libc::S_IFCHR | 0o777, dev_t)
	};
	assert_eq!(mknod_rc, 0);

	mktemp_template = mktemp_template.split_off("/dev/".len());

	let (dev_cuse, dev_cuse_closer) = DevCuse::new();

	let server_ready = sync::Arc::new(sync::Barrier::new(2));
	let server_thread = {
		let ready = sync::Arc::clone(&server_ready);
		thread::spawn(move || {
			use fuse::server::CuseConnectionBuilder;
			use fuse::server::basic::CuseServerBuilder;

			let devname = fuse::CuseDeviceName::from_bytes(&mktemp_template)
				.unwrap();
			let conn = CuseConnectionBuilder::new(dev_cuse, devname)
				.device_number(CUSE_DEV_MAJOR, CUSE_DEV_MINOR)
				.build()
				.unwrap();
			let srv = CuseServerBuilder::new(conn, handlers)
				.server_hooks(PrintHooks {})
				.build();
			ready.wait();

			let mut buf = fuse::io::ArrayBuffer::new();
			srv.serve(&mut buf).unwrap();
		})
	};

	server_ready.wait();
	let test_result = panic::catch_unwind(|| test_fn(&device_path));

	drop(dev_cuse_closer);
	let server_result = server_thread.join();

	if let Err(err) = test_result {
		panic::resume_unwind(err);
	} else {
		match server_result {
			Err(err) => panic::resume_unwind(err),
			Ok(_) => {
				//fuse_result.unwrap();
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
