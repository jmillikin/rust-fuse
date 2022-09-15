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

// use core::ffi::CStr;
use std::ffi::CStr;

use fuse::io::{InputStream, OutputStream, RecvError, SendError};

use crate::io::iovec::IoVec;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LibcError {
	code: i32,
}

impl LibcError {
	pub fn raw_os_error(&self) -> i32 {
		self.code
	}

	pub(crate) fn last_os_error() -> Self {
		Self::from_raw_os_error(errno())
	}

	pub(crate) fn from_raw_os_error(code: i32) -> Self {
		return Self { code };
	}
}

#[cfg(feature = "std")]
impl From<LibcError> for std::io::Error {
	fn from(err: LibcError) -> Self {
		std::io::Error::from_raw_os_error(err.code)
	}
}

struct Stream {
	fd: i32,
	enodev_is_eof: bool,
}

impl Drop for Stream {
	fn drop(&mut self) {
		unsafe {
			let _ = libc::close(self.fd);
		};
	}
}

impl Stream {
	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<LibcError>> {
		let buf_ptr = buf.as_mut_ptr() as *mut libc::c_void;
		let buf_len = buf.len();
		loop {
			let rc = unsafe { libc::read(self.fd, buf_ptr, buf_len) };
			if rc >= 0 {
				return Ok(rc as usize);
			}
			self.check_recv_err()?;
		}
	}

	#[cold]
	fn check_recv_err(&self) -> Result<(), RecvError<LibcError>> {
		match errno() {
			// The next request in the kernel buffer was interrupted before
			// it could be deleted. Try again.
			libc::ENOENT => Ok(()),

			// Interrupted by signal. Try again.
			libc::EINTR => Ok(()),

			// FUSE (but not CUSE) uses ENODEV to signal the clean shutdown of
			// the connection by the client.
			libc::ENODEV if self.enodev_is_eof => {
				Err(RecvError::ConnectionClosed)
			},

			err => Err(RecvError::Other(LibcError::from_raw_os_error(err))),
		}
	}

	fn send(&self, buf: &[u8]) -> Result<(), SendError<LibcError>> {
		let buf_ptr = buf.as_ptr() as *const libc::c_void;
		let buf_len = buf.len();
		loop {
			let write_rc = unsafe {
				libc::write(self.fd, buf_ptr, buf_len)
			};
			if write_rc == -1 {
				self.check_send_err()?;
				continue;
			}

			if write_rc == buf_len as isize {
				return Ok(());
			}
			let err = LibcError::from_raw_os_error(libc::EIO);
			return Err(SendError::Other(err));
		}
	}

	#[cold]
	fn check_send_err(&self) -> Result<(), SendError<LibcError>> {
		match errno() {
			libc::EINTR => Ok(()),
			libc::ENOENT => Err(SendError::NotFound),
			err => Err(SendError::Other(LibcError::from_raw_os_error(err))),
		}
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<LibcError>> {
		IoVec::borrow_array(bufs, |iovecs, bufs_len| {
			let iovecs_ptr = iovecs.as_ptr() as *const libc::iovec;
			loop {
				let write_rc = unsafe {
					libc::writev(self.fd, iovecs_ptr, N as i32)
				};
				if write_rc == -1 {
					self.check_send_err()?;
					continue;
				}

				if write_rc == bufs_len as isize {
					return Ok(());
				}
				let err = LibcError::from_raw_os_error(libc::EIO);
				return Err(SendError::Other(err));
			}
		})
	}
}

#[cfg(any(doc, not(target_os = "freebsd")))]
pub struct CuseStream {
	stream: Stream,
}

#[cfg(any(doc, not(target_os = "freebsd")))]
impl CuseStream {
	pub fn new() -> Result<CuseStream, LibcError> {
		Self::open(crate::DEV_CUSE)
	}

	pub fn open(dev_cuse: &CStr) -> Result<CuseStream, LibcError> {
		let path_ptr = dev_cuse.as_ptr() as *const libc::c_char;
		let open_rc = unsafe {
			libc::open(path_ptr, libc::O_RDWR | libc::O_CLOEXEC)
		};
		if open_rc == -1 {
			return Err(LibcError::last_os_error());
		}
		let stream = Stream {
			fd: open_rc,
			enodev_is_eof: false,
		};
		Ok(CuseStream { stream })
	}
}

#[cfg(any(doc, not(target_os = "freebsd")))]
impl InputStream for CuseStream {
	type Error = LibcError;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<LibcError>> {
		self.stream.recv(buf)
	}
}

#[cfg(any(doc, not(target_os = "freebsd")))]
impl OutputStream for CuseStream {
	type Error = LibcError;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<LibcError>> {
		self.stream.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<LibcError>> {
		self.stream.send_vectored(bufs)
	}
}

pub struct FuseStream {
	stream: Stream,
}

impl FuseStream {
	pub fn new() -> Result<FuseStream, LibcError> {
		Self::open(crate::DEV_FUSE)
	}

	pub fn open(dev_fuse: &CStr) -> Result<FuseStream, LibcError> {
		let path_ptr = dev_fuse.as_ptr() as *const libc::c_char;
		let open_rc = unsafe {
			libc::open(path_ptr, libc::O_RDWR | libc::O_CLOEXEC)
		};
		if open_rc == -1 {
			return Err(LibcError::last_os_error());
		}
		let stream = Stream {
			fd: open_rc,
			enodev_is_eof: true,
		};
		Ok(FuseStream { stream })
	}

	pub(crate) fn as_raw_fd(&self) -> i32 {
		self.stream.fd
	}
}

impl InputStream for FuseStream {
	type Error = LibcError;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<LibcError>> {
		self.stream.recv(buf)
	}
}

impl OutputStream for FuseStream {
	type Error = LibcError;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<LibcError>> {
		self.stream.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<LibcError>> {
		self.stream.send_vectored(bufs)
	}
}

#[cfg(target_os = "linux")]
fn errno() -> i32 {
	unsafe { *libc::__errno_location() }
}

#[cfg(target_os = "freebsd")]
fn errno() -> i32 {
	unsafe { *libc::__error() }
}
