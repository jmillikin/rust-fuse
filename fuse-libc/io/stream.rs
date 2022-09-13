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

use fuse::io::{InputStream, OutputStream, RecvError, SendError};

use crate::io::iovec::IoVec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LibcError {
	code: i32,
}

impl LibcError {
	pub fn raw_os_error(&self) -> i32 {
		self.code
	}

	#[allow(dead_code)]
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

pub struct LibcStream {
	fd: i32,
	enodev_is_eof: bool,
}

impl Drop for LibcStream {
	fn drop(&mut self) {
		unsafe {
			libc::close(self.fd)
		};
	}
}

impl LibcStream {
	pub(crate) fn dev_fuse() -> Result<Self, LibcError> {
		let path = b"/dev/fuse\0";
		let path_ptr = path.as_ptr() as *const libc::c_char;
		let open_rc = unsafe { libc::open(path_ptr, libc::O_RDWR) };
		if open_rc == -1 {
			return Err(LibcError::from_raw_os_error(open_rc));
		}
		Ok(Self {
			fd: open_rc,
			enodev_is_eof: true,
		})
	}

	#[allow(dead_code)]
	pub(crate) fn as_raw_fd(&self) -> i32 {
		self.fd
	}

	#[allow(dead_code)]
	pub(crate) fn fmt_raw_fd(&self, buf: &mut [u8; 32]) {
		let buf_ptr = buf.as_mut_ptr() as *mut libc::c_char;
		let format_ptr = b"%d\0".as_ptr() as *const libc::c_char;
		unsafe {
			libc::snprintf(buf_ptr, 32, format_ptr, self.fd)
		};
	}
}

impl InputStream for LibcStream {
	type Error = LibcError;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<LibcError>> {
		let buf_ptr = buf.as_mut_ptr() as *mut libc::c_void;
		let buf_len = buf.len();
		loop {
			let rc = unsafe { libc::read(self.fd, buf_ptr, buf_len) };
			if rc >= 0 {
				return Ok(rc as usize); // TOOD: .into()
			}
			match errno() {
				libc::EINTR => {
					// Interrupted by signal. Try again.
				},
				libc::ENODEV if self.enodev_is_eof => {
					return Err(RecvError::ConnectionClosed);
				},
				err_code => {
					let err = LibcError::from_raw_os_error(err_code);
					return Err(RecvError::Other(err));
				},
			}
		}
	}
}

impl OutputStream for LibcStream {
	type Error = LibcError;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<LibcError>> {
		let buf_ptr = buf.as_ptr() as *const libc::c_void;
		let buf_len = buf.len();
		let write_rc = unsafe { libc::write(self.fd, buf_ptr, buf_len) };
		if write_rc == -1 {
			match errno() {
				libc::ENOENT => {
					return Err(SendError::NotFound);
				},
				err_code => {
					let err = LibcError::from_raw_os_error(err_code);
					return Err(SendError::Other(err));
				},
			}
		}
		if write_rc < buf_len as isize {
			let err = LibcError::from_raw_os_error(libc::EIO);
			return Err(SendError::Other(err));
		}
		Ok(())
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<LibcError>> {

		IoVec::borrow_array(bufs, |iovecs, bufs_len| {
			let iovecs_ptr = iovecs.as_ptr() as *const libc::iovec;
			let write_rc = unsafe {
				libc::writev(self.fd, iovecs_ptr, N as i32)
			};
			if write_rc == -1 {
				match errno() {
					libc::ENOENT => {
						return Err(SendError::NotFound);
					},
					err_code => {
						let err = LibcError::from_raw_os_error(err_code);
						return Err(SendError::Other(err));
					},
				}
			}
			if write_rc < bufs_len as isize {
				let err = LibcError::from_raw_os_error(libc::EIO);
				return Err(SendError::Other(err));
			}
			Ok(())
		})
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
