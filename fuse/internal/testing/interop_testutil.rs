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

use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::{env, ffi, fs, io, panic, path, sync, thread};

use fuse::cuse;
use fuse::server;
use fuse::server::cuse_rpc;
use fuse::server::fuse_rpc;
use fuse::server::io::{SendError, RecvError};

#[cfg(target_os = "linux")]
pub use linux_errno as ErrorCode;

#[cfg(target_os = "freebsd")]
pub use freebsd_errno as ErrorCode;

struct PrintHooks {}

impl server::Hooks for PrintHooks {
	fn unknown_opcode(&self, request: server::Request) {
		println!(
			"\n[unknown_opcode]\n{:#?}\n{:?}\n",
			request.header(),
			request.as_slice(),
		);
	}

	fn unimplemented(&self, request: server::Request) {
		println!("\n[unimplemented]\n{:#?}", request.header());
	}

	fn request_error(
		&self,
		request: server::Request,
		err: server::RequestError,
	) {
		println!("\n[request_error]\n{:#?}", request.header());
		println!("{:#?}", err);
	}
}

#[cfg(target_os = "linux")]
type DevFuse = fuse_linux::FuseServerSocket;

#[cfg(target_os = "freebsd")]
type DevFuse = fuse_libc::FuseServerSocket;

pub trait TestDev: cuse_rpc::Handlers<DevCuse> {
	#[allow(unused)]
	fn cuse_init(
		init_request: &fuse::CuseInitRequest,
		init_response: &mut fuse::CuseInitResponse,
	) {
	}
}

pub trait TestFS: fuse_rpc::Handlers<DevFuse> {
	#[allow(unused)]
	fn fuse_init(
		init_request: &fuse::FuseInitRequest,
		init_response: &mut fuse::FuseInitResponse,
	) {
	}

	fn mount_subtype(&self) -> ffi::CString {
		ffi::CString::new("rust_fuse_test").unwrap()
	}

	#[cfg(target_os = "linux")]
	fn mount_type(&self) -> &'static fuse::os::linux::MountType {
		fuse::os::linux::MountType::FUSE
	}

	#[cfg(target_os = "linux")]
	fn mount_source(&self) -> ffi::CString {
		ffi::CString::new("rust_fuse_test").unwrap()
	}

	#[cfg(target_os = "freebsd")]
	#[allow(unused)]
	fn freebsd_mount_options(
		&self,
		freebsd_options: &mut fuse::os::freebsd::MountOptions,
	) {
	}

	#[cfg(target_os = "linux")]
	#[allow(unused)]
	fn linux_mount_options(
		&self,
		mount_options: &mut fuse::os::linux::MountOptions,
	) {
	}
}

pub fn fuse_interop_test<H: TestFS + Send + 'static>(
	handlers: H,
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

	let dev_fuse;

	#[cfg(target_os = "linux")]
	{
		let mut mount_options = fuse::os::linux::MountOptions::new();

		let mount_source = handlers.mount_source();
		let mount_subtype = handlers.mount_subtype();
		mount_options.set_mount_source(
			fuse::os::linux::MountSource::new(&mount_source).unwrap(),
		);
		mount_options.set_mount_type(handlers.mount_type());
		mount_options.set_subtype(Some(
			fuse::os::linux::FuseSubtype::new(&mount_subtype).unwrap(),
		));

		handlers.linux_mount_options(&mut mount_options);

		dev_fuse = fuse_linux::mount(&mount_cstr, mount_options).unwrap();
	}

	#[cfg(target_os = "freebsd")]
	{
		let mut mount_options = fuse::os::freebsd::MountOptions::new();

		let mount_subtype = handlers.mount_subtype();
		mount_options.set_subtype(Some(
			fuse::os::freebsd::FuseSubtype::new(&mount_subtype).unwrap(),
		));

		handlers.freebsd_mount_options(&mut mount_options);

		dev_fuse = fuse_libc::os::freebsd::mount(&mount_cstr, mount_options)
			.unwrap();
	}

	let server_ready = sync::Arc::new(sync::Barrier::new(2));
	let server_thread = {
		let ready = sync::Arc::clone(&server_ready);
		thread::spawn(move || {
			let builder = fuse_rpc::ServerBuilder::new(dev_fuse, handlers);
			let srv = builder
				.server_hooks(Box::new(PrintHooks {}))
				.fuse_init_fn(|req, resp| {
					H::fuse_init(req, resp);
				})
				.unwrap();
			ready.wait();

			srv.serve().unwrap();
		})
	};

	server_ready.wait();
	let test_result = panic::catch_unwind(|| test_fn(&mount_path));

	let unmount_rc = unsafe {
		#[cfg(target_os = "linux")]
		let unmount_rc = libc::umount(mount_cstr.as_ptr());

		#[cfg(target_os = "freebsd")]
		let unmount_rc = libc::unmount(mount_cstr.as_ptr(), 0);

		if unmount_rc == -1 {
			#[cfg(target_os = "linux")]
			libc::umount2(mount_cstr.as_ptr(), libc::MNT_FORCE);

			#[cfg(target_os = "freebsd")]
			libc::unmount(mount_cstr.as_ptr(), libc::MNT_FORCE);
		}
		unmount_rc
	};

	let server_result = server_thread.join();

	if let Err(err) = test_result {
		panic::resume_unwind(err);
	} else {
		match server_result {
			Err(err) => panic::resume_unwind(err),
			Ok(_) => {
				//fuse_result.unwrap();
				assert_eq!(unmount_rc, 0);
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

impl server::io::CuseSocket for DevCuse {}

impl server::io::Socket for DevCuse {
	type Error = io::Error;

	fn send(
		&self,
		buf: fuse::io::SendBuf,
	) -> Result<(), SendError<io::Error>> {
		use std::io::Write;
		let buf = buf.to_vec().unwrap();
		let write_size = match Write::write(&mut &self.dev_cuse, &buf) {
			Err(err) => return Err(SendError::Other(err)),
			Ok(x) => x,
		};
		if write_size < buf.len() {
			return Err(SendError::Other(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			)));
		}
		Ok(())
	}

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<io::Error>> {
		use std::io::Read;
		use std::os::unix::io::AsRawFd;

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
				let err = io::ErrorKind::ConnectionReset;
				return Err(RecvError::ConnectionClosed(err.into()));
			}

			if (poll_fds[0].revents & libc::POLLIN) == 0 {
				continue;
			}

			match Read::read(&mut &self.dev_cuse, buf) {
				Ok(size) => return Ok(size),
				Err(err) => match err.raw_os_error() {
					Some(libc::ENOENT) => {
						// The next request in the kernel buffer was interrupted before
						// it could be deleted. Try again.
					},
					Some(libc::EINTR) => {
						// Interrupted by signal. Try again.
					},
					_ => return Err(RecvError::Other(err)),
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

pub fn cuse_interop_test<H: TestDev + Send + 'static>(
	handlers: H,
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
	let device_path = path::Path::new(ffi::OsStr::from_bytes(&mktemp_template))
		.to_path_buf();

	#[cfg(target_os = "linux")]
	{
		let devpath_cstr = ffi::CString::new(mktemp_template.clone()).unwrap();
		let mknod_rc = unsafe {
			let dev_t = libc::makedev(CUSE_DEV_MAJOR, CUSE_DEV_MINOR);
			libc::mknod(devpath_cstr.as_ptr(), libc::S_IFCHR | 0o777, dev_t)
		};
		assert_eq!(mknod_rc, 0);
	}

	mktemp_template = mktemp_template.split_off("/dev/".len());

	let (dev_cuse, dev_cuse_closer) = DevCuse::new();

	let server_ready = sync::Arc::new(sync::Barrier::new(2));
	let server_thread = {
		let ready = sync::Arc::clone(&server_ready);
		thread::spawn(move || {
			let devname = cuse::DeviceName::from_bytes(&mktemp_template)
				.unwrap();
			let srv = cuse_rpc::ServerBuilder::new(dev_cuse, handlers)
				.device_number(cuse::DeviceNumber::new(
					CUSE_DEV_MAJOR,
					CUSE_DEV_MINOR,
				))
				.server_hooks(Box::new(PrintHooks {}))
				.cuse_init_fn(devname, |req, resp| {
					H::cuse_init(req, resp);
				})
				.unwrap();
			ready.wait();

			use fuse::server::ServerError;
			fn is_conn_reset(err: &ServerError<io::Error>) -> bool {
				if let ServerError::RecvError(err) = err {
					if err.kind() == io::ErrorKind::ConnectionReset {
						return true;
					}
				}
				false
			}

			match srv.serve() {
				Ok(_) => {},
				Err(err) if is_conn_reset(&err) => {},
				err => err.unwrap(),
			}
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
