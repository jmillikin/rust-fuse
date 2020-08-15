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

use core::convert::TryInto;
use core::mem::{self, MaybeUninit};
use std::io::{self, IoSlice, Read, Write};

use crate::error::{Error, ErrorKind};

pub trait Channel: Sized {
	fn send(&self, buf: &[u8]) -> Result<(), Error>;

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), Error>;

	fn receive(&self, buf: &mut [u8]) -> Result<usize, Error>;

	fn try_clone(&self) -> Result<Self, Error>;
}

pub(crate) struct FileChannel {
	file: std::fs::File,
}

impl FileChannel {
	pub(crate) fn new(file: std::fs::File) -> Self {
		Self { file }
	}
}

impl Channel for FileChannel {
	fn send(&self, buf: &[u8]) -> Result<(), Error> {
		let write_size =
			Write::write(&mut &self.file, buf).map_err(convert_error)?;
		if write_size < buf.len() {
			return Err(Error(ErrorKind::IncompleteWrite));
		}
		Ok(())
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), Error> {
		let mut bufs_len: usize = 0;
		let io_slices: &[IoSlice] = {
			let mut uninit_bufs: [MaybeUninit<IoSlice>; N] =
				unsafe { MaybeUninit::uninit().assume_init() };
			for ii in 0..N {
				bufs_len += bufs[ii].len();
				uninit_bufs[ii] = MaybeUninit::new(IoSlice::new(bufs[ii]));
			}
			unsafe { mem::transmute::<_, &[IoSlice; N]>(&uninit_bufs) }
		};

		let write_size = Write::write_vectored(&mut &self.file, io_slices)
			.map_err(convert_error)?;
		if write_size < bufs_len {
			return Err(Error(ErrorKind::IncompleteWrite));
		}
		Ok(())
	}

	fn receive(&self, buf: &mut [u8]) -> Result<usize, Error> {
		Read::read(&mut &self.file, buf).map_err(convert_error)
	}

	fn try_clone(&self) -> Result<Self, Error> {
		Ok(Self {
			file: self.file.try_clone().map_err(convert_error)?,
		})
	}
}

fn convert_error(err: io::Error) -> Error {
	if let Some(os_err) = err.raw_os_error() {
		if let Ok(os_err) = os_err.try_into() {
			if let Some(err_code) = core::num::NonZeroU16::new(os_err) {
				return Error::new(err_code.into());
			}
		}
	}
	Error(ErrorKind::Unknown)
}
