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

#[cfg(feature = "nightly_impl_channel")]
use core::mem::{self, MaybeUninit};

#[cfg(feature = "std")]
use std::io::{self, IoSlice, Read, Write};

use crate::error::{Error, ErrorCode};

#[cfg(any(doc, feature = "nightly_impl_channel"))]
pub trait Channel {
	type Error: ChannelError;

	fn send(&self, buf: &[u8]) -> Result<(), Self::Error>;

	#[cfg_attr(doc, doc(cfg(feature = "nightly_impl_channel")))]
	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), Self::Error>;

	fn receive(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

#[cfg(not(feature = "nightly_impl_channel"))]
pub(crate) mod private {
	pub trait ChannelNoConstGenerics<Error> {
		fn send_vectored_2(&self, bufs: &[&[u8]; 2]) -> Result<(), Error>;
		fn send_vectored_3(&self, bufs: &[&[u8]; 3]) -> Result<(), Error>;
		fn send_vectored_5(&self, bufs: &[&[u8]; 5]) -> Result<(), Error>;
	}
}

#[cfg(not(any(doc, feature = "nightly_impl_channel")))]
pub trait Channel:
	private::ChannelNoConstGenerics<<Self as Channel>::Error>
{
	type Error: ChannelError;

	fn send(&self, buf: &[u8]) -> Result<(), Self::Error>;

	fn receive(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait ChannelError: From<Error> {
	fn error_code(&self) -> Option<ErrorCode>;
}

#[cfg(feature = "std")]
pub(crate) struct FileChannel {
	file: std::fs::File,
}

#[cfg(feature = "std")]
impl FileChannel {
	pub(crate) fn new(file: std::fs::File) -> Self {
		Self { file }
	}

	pub(crate) fn try_clone(&self) -> Result<Self, io::Error> {
		Ok(Self {
			file: self.file.try_clone()?,
		})
	}
}

#[cfg(feature = "std")]
#[cfg(not(feature = "nightly_impl_channel"))]
impl private::ChannelNoConstGenerics<io::Error> for FileChannel {
	fn send_vectored_2(&self, bufs: &[&[u8]; 2]) -> Result<(), io::Error> {
		let mut bufs_len: usize = 0;
		for buf in bufs {
			bufs_len += buf.len();
		}
		let write_size = Write::write_vectored(
			&mut &self.file,
			&[IoSlice::new(bufs[0]), IoSlice::new(bufs[1])],
		)?;
		if write_size < bufs_len {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}

	fn send_vectored_3(&self, bufs: &[&[u8]; 3]) -> Result<(), io::Error> {
		let mut bufs_len: usize = 0;
		for buf in bufs {
			bufs_len += buf.len();
		}
		let write_size = Write::write_vectored(
			&mut &self.file,
			&[
				IoSlice::new(bufs[0]),
				IoSlice::new(bufs[1]),
				IoSlice::new(bufs[2]),
			],
		)?;
		if write_size < bufs_len {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}

	fn send_vectored_5(&self, bufs: &[&[u8]; 5]) -> Result<(), io::Error> {
		let mut bufs_len: usize = 0;
		for buf in bufs {
			bufs_len += buf.len();
		}
		let write_size = Write::write_vectored(
			&mut &self.file,
			&[
				IoSlice::new(bufs[0]),
				IoSlice::new(bufs[1]),
				IoSlice::new(bufs[2]),
				IoSlice::new(bufs[3]),
				IoSlice::new(bufs[4]),
			],
		)?;
		if write_size < bufs_len {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}
}

#[cfg(feature = "std")]
impl Channel for FileChannel {
	type Error = io::Error;

	fn send(&self, buf: &[u8]) -> Result<(), io::Error> {
		let write_size = Write::write(&mut &self.file, buf)?;
		if write_size < buf.len() {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}

	#[cfg(any(doc, feature = "nightly_impl_channel"))]
	
	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), io::Error> {
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

		let write_size = Write::write_vectored(&mut &self.file, io_slices)?;
		if write_size < bufs_len {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"incomplete send",
			));
		}
		Ok(())
	}

	fn receive(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
		Read::read(&mut &self.file, buf)
	}
}

#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
impl ChannelError for io::Error {
	fn error_code(&self) -> Option<ErrorCode> {
		if let Some(os_err) = self.raw_os_error() {
			if let Ok(os_err) = os_err.try_into() {
				if let Some(err_code) = core::num::NonZeroU16::new(os_err) {
					return Some(err_code.into());
				}
			}
		}
		None
	}
}
