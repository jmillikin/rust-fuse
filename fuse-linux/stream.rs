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

use core::mem::{self, MaybeUninit};
// use core::ffi::CStr;
use std::ffi::CStr;

use fuse::io::{InputStream, OutputStream, RecvError, SendError};
use linux_errno::{self as errno, Error};

use crate::sys;

struct Stream {
	fd: i32,
	enodev_is_eof: bool,
}

impl Drop for Stream {
	fn drop(&mut self) {
		unsafe {
			let _ = sys::close(self.fd);
		}
	}
}

impl Stream {
	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Error>> {
		loop {
			match unsafe { sys::read(self.fd, buf) } {
				Ok(read_size) => return Ok(read_size),
				Err(err) => self.check_recv_err(err)?,
			}
		}
	}

	#[cold]
	fn check_recv_err(&self, err: Error) -> Result<(), RecvError<Error>> {
		match err {
			// The next request in the kernel buffer was interrupted before
			// it could be deleted. Try again.
			errno::ENOENT => Ok(()),

			// Interrupted by signal. Try again.
			errno::EINTR => Ok(()),

			// FUSE (but not CUSE) uses ENODEV to signal the clean shutdown of
			// the connection by the client.
			errno::ENODEV if self.enodev_is_eof => {
				Err(RecvError::ConnectionClosed)
			},

			_ => Err(RecvError::Other(err)),
		}
	}

	fn send(&self, buf: &[u8]) -> Result<(), SendError<Error>> {
		loop {
			match unsafe { sys::write(self.fd, buf) } {
				Ok(write_size) => {
					if write_size == buf.len() {
						return Ok(());
					}
					return Err(SendError::Other(errno::EIO));
				},
				Err(err) => self.check_send_err(err)?,
			};
		}
	}

	#[cold]
	fn check_send_err(&self, err: Error) -> Result<(), SendError<Error>> {
		match err {
			errno::EINTR => Ok(()),
			errno::ENOENT => Err(SendError::NotFound),
			_ => Err(SendError::Other(err)),
		}
	}

	fn send_vectored<'a, const N: usize>(
		&self,
		bufs: &[&'a [u8]; N],
	) -> Result<(), SendError<Error>> {
		borrow_iovec_array(bufs, |iovecs, bufs_len| {
			loop {
				match unsafe { sys::writev(self.fd, iovecs) } {
					Ok(write_size) => {
						if write_size == bufs_len {
							return Ok(());
						}
						return Err(SendError::Other(errno::EIO));
					},
					Err(err) => self.check_send_err(err)?,
				}
			}
		})
	}
}

pub struct CuseStream {
	stream: Stream,
}

impl CuseStream {
	pub fn new() -> Result<CuseStream, Error> {
		Self::open(crate::DEV_CUSE)
	}

	pub fn open(dev_cuse: &CStr) -> Result<CuseStream, Error> {
		let fd = unsafe {
			sys::open(sys::AT_FDCWD, dev_cuse, sys::O_RDWR | sys::O_CLOEXEC, 0)?
		};
		let stream = Stream {
			fd,
			enodev_is_eof: false,
		};
		Ok(CuseStream { stream })
	}
}

impl InputStream for CuseStream {
	type Error = Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Error>> {
		self.stream.recv(buf)
	}
}

impl OutputStream for CuseStream {
	type Error = Error;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<Error>> {
		self.stream.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<Error>> {
		self.stream.send_vectored(bufs)
	}
}

pub struct FuseStream {
	stream: Stream,
}

impl FuseStream {
	pub fn new() -> Result<FuseStream, Error> {
		Self::open(crate::DEV_FUSE)
	}

	pub fn open(dev_fuse: &CStr) -> Result<FuseStream, Error> {
		let fd = unsafe {
			sys::open(sys::AT_FDCWD, dev_fuse, sys::O_RDWR | sys::O_CLOEXEC, 0)?
		};
		let stream = Stream {
			fd,
			enodev_is_eof: true,
		};
		Ok(FuseStream { stream })
	}

	pub(crate) fn as_raw_fd(&self) -> i32 {
		self.stream.fd
	}
}

impl InputStream for FuseStream {
	type Error = Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Error>> {
		self.stream.recv(buf)
	}
}

impl OutputStream for FuseStream {
	type Error = Error;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<Error>> {
		self.stream.send(buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<Error>> {
		self.stream.send_vectored(bufs)
	}
}

fn borrow_iovec_array<'a, T, const N: usize>(
	bufs: &[&'a [u8]; N],
	f: impl FnOnce(&[sys::IoVec<'a>; N], usize) -> T,
) -> T {
	let mut bufs_len: usize = 0;
	let mut uninit_bufs: [MaybeUninit<sys::IoVec<'a>>; N] = unsafe {
		MaybeUninit::uninit().assume_init()
	};
	for ii in 0..N {
		let buf = bufs[ii];
		bufs_len += buf.len();
		uninit_bufs[ii] = MaybeUninit::new(sys::IoVec::borrow(buf));
	}
	let iovecs = unsafe {
		mem::transmute::<_, &[sys::IoVec<'a>; N]>(&uninit_bufs)
	};
	f(iovecs, bufs_len)
}
