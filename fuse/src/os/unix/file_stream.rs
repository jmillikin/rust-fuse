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

use std::fs::{self, File};
use std::io;

use crate::error::ErrorCode;
use crate::io::{InputStream, OutputStream, RecvError, SendError};

fn file_recv(
	mut file: &File,
	buf: &mut [u8],
	enodev_is_eof: bool,
) -> Result<usize, RecvError<io::Error>> {
	use std::io::Read;

	loop {
		match Read::read(&mut file, buf) {
			Ok(size) => return Ok(size),
			Err(err) => match err.raw_os_error() {
				Some(ErrorCode::ENOENT_I32) => {
					// The next request in the kernel buffer was interrupted before
					// it could be deleted. Try again.
				},
				Some(ErrorCode::EINTR_I32) => {
					// Interrupted by signal. Try again.
				},
				Some(ErrorCode::ENODEV_I32) => {
					if enodev_is_eof {
						return Err(RecvError::ConnectionClosed);
					}
					return Err(RecvError::Other(err));
				},
				_ => return Err(RecvError::Other(err)),
			},
		}
	}
}

fn file_send(mut file: &File, buf: &[u8]) -> Result<(), SendError<io::Error>> {
	use std::io::Write;

	let write_size = match Write::write(&mut file, buf) {
		Ok(x) => x,
		Err(err) => return Err(send_error(err)),
	};
	if write_size < buf.len() {
		return Err(SendError::Other(io::ErrorKind::WriteZero.into()));
	}
	Ok(())
}

fn file_send_vectored<const N: usize>(
	mut file: &File,
	bufs: &[&[u8]; N],
) -> Result<(), SendError<io::Error>> {
	use std::io::Write;

	let mut bufs_len: usize = 0;
	let mut uninit_bufs: [MaybeUninit<io::IoSlice>; N] =
		unsafe { MaybeUninit::uninit().assume_init() };
	for ii in 0..N {
		bufs_len += bufs[ii].len();
		uninit_bufs[ii] = MaybeUninit::new(io::IoSlice::new(bufs[ii]));
	}
	let io_slices: &[io::IoSlice] = unsafe {
		mem::transmute::<_, &[io::IoSlice; N]>(&uninit_bufs)
	};

	let write_size = match Write::write_vectored(&mut file, io_slices) {
		Ok(x) => x,
		Err(err) => return Err(send_error(err)),
	};
	if write_size < bufs_len {
		return Err(SendError::Other(io::ErrorKind::WriteZero.into()));
	}
	Ok(())
}

fn send_error(err: io::Error) -> SendError<io::Error> {
	match err.raw_os_error() {
		Some(ErrorCode::ENOENT_I32) => SendError::NotFound,
		_ => SendError::Other(err),
	}
}

pub struct DevCuse(File);

impl DevCuse {
	pub fn new() -> Result<Self, io::Error> {
		let dev_cuse = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open("/dev/cuse")?;
		Ok(Self(dev_cuse))
	}
}

impl InputStream for DevCuse {
	type Error = io::Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<io::Error>> {
		file_recv(&self.0, buf, false)
	}
}

impl OutputStream for DevCuse {
	type Error = io::Error;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<io::Error>> {
		file_send(&self.0, buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<io::Error>> {
		file_send_vectored(&self.0, bufs)
	}
}

pub struct DevFuse(File);

impl DevFuse {
	#[allow(dead_code)]
	pub(crate) fn from_file(file: File) -> Self {
		Self(file)
	}
}

impl InputStream for DevFuse {
	type Error = io::Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<io::Error>> {
		file_recv(&self.0, buf, true)
	}
}

impl OutputStream for DevFuse {
	type Error = io::Error;

	fn send(&self, buf: &[u8]) -> Result<(), SendError<io::Error>> {
		file_send(&self.0, buf)
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), SendError<io::Error>> {
		file_send_vectored(&self.0, bufs)
	}
}
