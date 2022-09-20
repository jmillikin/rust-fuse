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

use fuse::io::{
	ServerRecvError as RecvError,
	ServerSendError as SendError,
};
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

struct Socket {
	fd: i32,
	enodev_is_eof: bool,
}

impl Drop for Socket {
	fn drop(&mut self) {
		unsafe {
			let _ = libc::close(self.fd);
		};
	}
}

impl Socket {
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
				let err = LibcError::from_raw_os_error(libc::ENODEV);
				Err(RecvError::ConnectionClosed(err))
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
		let errno = errno();
		if errno == libc::EINTR {
			return Ok(());
		}
		let err = LibcError::from_raw_os_error(errno);
		Err(match errno {
			libc::ENOENT => SendError::NotFound(err),
			_ => SendError::Other(err),
		})
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
pub struct CuseServerSocket {
	socket: Socket,
}

#[cfg(any(doc, not(target_os = "freebsd")))]
impl CuseServerSocket {
	pub fn new() -> Result<CuseServerSocket, LibcError> {
		Self::open(crate::DEV_CUSE)
	}

	pub fn open(dev_cuse: &CStr) -> Result<CuseServerSocket, LibcError> {
		let path_ptr = dev_cuse.as_ptr() as *const libc::c_char;
		let open_rc = unsafe {
			libc::open(path_ptr, libc::O_RDWR | libc::O_CLOEXEC)
		};
		if open_rc == -1 {
			return Err(LibcError::last_os_error());
		}
		let socket = Socket {
			fd: open_rc,
			enodev_is_eof: false,
		};
		Ok(CuseServerSocket { socket })
	}
}

#[cfg(any(doc, not(target_os = "freebsd")))]
impl fuse::io::CuseServerSocket for CuseServerSocket {}

#[cfg(any(doc, not(target_os = "freebsd")))]
impl fuse::io::ServerSocket for CuseServerSocket {
	type Error = LibcError;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<LibcError>> {
		self.socket.recv(buf)
	}

	fn send(&self, buf: &[u8]) -> Result<(), SendError<LibcError>> {
		self.socket.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<LibcError>> {
		self.socket.send_vectored(bufs)
	}
}

pub struct FuseServerSocket {
	socket: Socket,
}

impl FuseServerSocket {
	pub fn new() -> Result<FuseServerSocket, LibcError> {
		Self::open(crate::DEV_FUSE)
	}

	pub fn open(dev_fuse: &CStr) -> Result<FuseServerSocket, LibcError> {
		let path_ptr = dev_fuse.as_ptr() as *const libc::c_char;
		let open_rc = unsafe {
			libc::open(path_ptr, libc::O_RDWR | libc::O_CLOEXEC)
		};
		if open_rc == -1 {
			return Err(LibcError::last_os_error());
		}
		let socket = Socket {
			fd: open_rc,
			enodev_is_eof: true,
		};
		Ok(FuseServerSocket { socket })
	}

	pub fn fuse_device_fd(&self) -> u32 {
		self.socket.fd as u32
	}
}

impl fuse::io::FuseServerSocket for FuseServerSocket {}

impl fuse::io::ServerSocket for FuseServerSocket {
	type Error = LibcError;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<LibcError>> {
		self.socket.recv(buf)
	}

	fn send(&self, buf: &[u8]) -> Result<(), SendError<LibcError>> {
		self.socket.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<LibcError>> {
		self.socket.send_vectored(bufs)
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
