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

use core::num::NonZeroU16;
use std::ffi::CStr;
use std::io::{self, IoSlice, Read, Write};
use std::mem::{align_of, size_of};
use std::pin::Pin;

use crate::internal::fuse_kernel;

#[cfg(test)]
#[path = "fuse_io_test.rs"]
mod fuse_io_test;

pub(crate) trait Channel {
	fn read(&self, buf: &mut [u8]) -> io::Result<usize>;
	fn write(&self, buf: &[u8]) -> io::Result<()>;
	fn write_vectored(&self, bufs: &[io::IoSlice]) -> io::Result<()>;
}

pub(crate) struct FileChannel {
	file: std::fs::File,
}

impl FileChannel {
	pub(crate) fn new(file: std::fs::File) -> Self {
		Self { file }
	}

	pub(crate) fn try_clone(&self) -> io::Result<Self> {
		Ok(Self {
			file: self.file.try_clone()?,
		})
	}
}

impl Channel for FileChannel {
	fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
		Read::read(&mut &self.file, buf)
	}

	fn write(&self, buf: &[u8]) -> io::Result<()> {
		let write_size = Write::write(&mut &self.file, buf)?;
		// TODO: check if write_size < buf.len()
		Ok(())
	}

	fn write_vectored(&self, bufs: &[io::IoSlice]) -> io::Result<()> {
		let write_size = Write::write_vectored(&mut &self.file, bufs)?;
		// TODO: check if write_size < bufs.sum(|x| x.len())
		Ok(())
	}
}

pub(crate) trait AlignedBuffer {
	fn get(&self) -> &[u8];
	fn get_mut(&mut self) -> &mut [u8];
}

pub(crate) fn aligned_slice<Buf: AlignedBuffer>(
	buf: &Buf,
	size: usize,
) -> AlignedSlice {
	// TODO: validate size
	AlignedSlice {
		buf: &buf.get()[0..size],
	}
}

pub(crate) struct MinReadBuffer {
	_align: [u64; 0],
	buf: [u8; fuse_kernel::FUSE_MIN_READ_BUFFER as usize],
}

impl MinReadBuffer {
	pub(crate) fn new() -> Self {
		Self {
			_align: [0; 0],
			buf: [0; fuse_kernel::FUSE_MIN_READ_BUFFER as usize],
		}
	}

	#[cfg(test)]
	pub(crate) fn borrow(&self) -> AlignedSlice {
		AlignedSlice { buf: &self.buf }
	}
}

impl AlignedBuffer for MinReadBuffer {
	fn get(&self) -> &[u8] {
		&self.buf
	}

	fn get_mut(&mut self) -> &mut [u8] {
		&mut self.buf
	}
}

pub(crate) struct AlignedSlice<'a> {
	buf: &'a [u8],
}

impl<'a> AlignedSlice<'a> {
	pub fn get(self) -> &'a [u8] {
		self.buf
	}
}

pub(crate) struct AlignedVec {
	pinned: Pin<Box<[u8]>>,
	offset: usize,
}

impl AlignedVec {
	pub(crate) fn new(size: usize) -> Self {
		let mut vec = Vec::with_capacity(size + 7);
		vec.resize(size + 7, 0u8);
		let pinned = Pin::new(vec.into_boxed_slice());
		let offset = pinned.as_ptr().align_offset(align_of::<u64>());
		Self { pinned, offset }
	}
}

impl AlignedBuffer for AlignedVec {
	fn get(&self) -> &[u8] {
		if self.offset == 0 {
			return &self.pinned;
		}
		let (_, aligned) = self.pinned.split_at(self.offset);
		aligned
	}

	fn get_mut(&mut self) -> &mut [u8] {
		if self.offset == 0 {
			return &mut self.pinned;
		}
		let (_, aligned) = self.pinned.split_at_mut(self.offset);
		aligned
	}
}

pub(crate) trait DecodeRequest<'a>: Sized {
	fn decode_request(decoder: RequestDecoder<'a>) -> io::Result<Self>;
}

pub(crate) struct RequestDecoder<'a> {
	buf: &'a [u8],
	header: &'a fuse_kernel::fuse_in_header,
	version: crate::ProtocolVersion,
	consumed: u32,
}

impl<'a> RequestDecoder<'a> {
	pub(crate) fn new(
		buf: AlignedSlice<'a>,
		version: crate::ProtocolVersion,
	) -> io::Result<Self> {
		let buf = buf.get();
		if buf.len() < size_of::<fuse_kernel::fuse_in_header>() {
			return Err(io::ErrorKind::UnexpectedEof.into());
		}

		let header: &'a fuse_kernel::fuse_in_header =
			unsafe { &*(buf.as_ptr() as *const fuse_kernel::fuse_in_header) };

		let buf_len: u32;
		if size_of::<usize>() > size_of::<u32>() {
			if buf.len() > u32::MAX as usize {
				buf_len = u32::MAX;
			} else {
				buf_len = buf.len() as u32;
			}
		} else {
			buf_len = buf.len() as u32;
		}
		if buf_len < header.len {
			return Err(io::ErrorKind::UnexpectedEof.into());
		}

		Ok(RequestDecoder {
			buf,
			header,
			version,
			consumed: size_of::<fuse_kernel::fuse_in_header>() as u32,
		})
	}

	pub(crate) fn header(&self) -> &'a fuse_kernel::fuse_in_header {
		self.header
	}

	pub(crate) fn version(&self) -> crate::ProtocolVersion {
		self.version
	}

	fn consume(&self, len: u32) -> io::Result<u32> {
		let new_consumed: u32;
		let eof: bool;
		match self.consumed.checked_add(len) {
			Some(x) => {
				new_consumed = x;
				eof = new_consumed > self.header.len;
			},
			None => {
				new_consumed = 0;
				eof = true;
			},
		}
		if eof {
			return Err(io::ErrorKind::UnexpectedEof.into());
		}
		debug_assert!(new_consumed <= self.header.len);
		Ok(new_consumed)
	}

	pub(crate) fn peek_sized<T: Sized>(&self) -> io::Result<&'a T> {
		if size_of::<usize>() > size_of::<u32>() {
			debug_assert!(size_of::<T>() < u32::MAX as usize);
		}
		self.consume(size_of::<T>() as u32)?;
		let out: &'a T = unsafe {
			let out_p = self.buf.as_ptr().add(self.consumed as usize);
			&*(out_p as *const T)
		};
		Ok(out)
	}

	pub(crate) fn next_sized<T: Sized>(&mut self) -> io::Result<&'a T> {
		let out = self.peek_sized()?;
		self.consumed = self.consume(size_of::<T>() as u32)?;
		Ok(out)
	}

	pub(crate) fn next_bytes(&mut self, len: u32) -> io::Result<&'a [u8]> {
		let new_consumed = self.consume(len)?;
		let (_, start) = self.buf.split_at(self.consumed as usize);
		let (out, _) = start.split_at(len as usize);
		self.consumed = new_consumed;
		Ok(out)
	}

	pub(crate) fn next_cstr(&mut self) -> io::Result<&'a CStr> {
		for off in self.consumed..self.header.len {
			if self.buf[off as usize] == 0 {
				let len = off - self.consumed;
				let buf = self.next_bytes(len + 1)?;
				return Ok(unsafe { CStr::from_bytes_with_nul_unchecked(buf) });
			}
		}
		Err(io::ErrorKind::UnexpectedEof.into())
	}
}

pub(crate) trait EncodeResponse {
	fn encode_response<Chan: Channel>(
		&self,
		enc: ResponseEncoder<Chan>,
	) -> io::Result<()>;
}

pub(crate) struct ResponseEncoder<'a, Chan> {
	channel: &'a Chan,
	request_id: u64,
	version: crate::ProtocolVersion,
}

impl<'a, Chan> ResponseEncoder<'a, Chan> {
	pub(crate) fn new(
		channel: &'a Chan,
		request_id: u64,
		version: crate::ProtocolVersion,
	) -> Self {
		Self {
			channel,
			request_id,
			version,
		}
	}

	pub(crate) fn version(&self) -> crate::ProtocolVersion {
		self.version
	}
}

impl<Chan: Channel> ResponseEncoder<'_, Chan> {
	pub(crate) fn encode_error(self, err: NonZeroU16) -> io::Result<()> {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: -(err.get() as i32),
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.channel.write(out_hdr_buf)
	}

	pub(crate) fn encode_sized<T: Sized>(self, t: &T) -> io::Result<()> {
		let bytes: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(t as *const T) as *const u8,
				size_of::<T>(),
			)
		};
		self.encode_bytes(bytes)
	}

	pub(crate) fn encode_sized_bytes<T: Sized>(
		self,
		bytes_1: &[u8],
		t: &T,
	) -> io::Result<()> {
		let bytes_2: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(t as *const T) as *const u8,
				size_of::<T>(),
			)
		};
		self.encode_bytes_2(bytes_1, bytes_2)
	}

	pub(crate) fn encode_sized_sized<T1: Sized, T2: Sized>(
		self,
		t_1: &T1,
		t_2: &T2,
	) -> io::Result<()> {
		let bytes_1: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(t_1 as *const T1) as *const u8,
				size_of::<T1>(),
			)
		};
		let bytes_2: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(t_2 as *const T2) as *const u8,
				size_of::<T2>(),
			)
		};
		self.encode_bytes_2(bytes_1, bytes_2)
	}

	pub(crate) fn encode_header_only(self) -> io::Result<()> {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.channel.write(out_hdr_buf)
	}

	pub(crate) fn encode_bytes(self, bytes: &[u8]) -> io::Result<()> {
		let mut len = size_of::<fuse_kernel::fuse_out_header>();

		match len.checked_add(bytes.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes.len()),
		}

		if size_of::<usize>() > size_of::<u32>() {
			if len > u32::MAX as usize {
				panic!("{} overflows u32", len);
			}
		}

		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};
		self.channel
			.write_vectored(&[IoSlice::new(out_hdr_buf), IoSlice::new(bytes)])
	}

	pub(crate) fn encode_bytes_2(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
	) -> io::Result<()> {
		let mut len = size_of::<fuse_kernel::fuse_out_header>();

		match len.checked_add(bytes_1.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_1.len()),
		}
		match len.checked_add(bytes_2.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_2.len()),
		}

		if size_of::<usize>() > size_of::<u32>() {
			if len > u32::MAX as usize {
				panic!("{} overflows u32", len);
			}
		}

		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};
		self.channel.write_vectored(&[
			IoSlice::new(out_hdr_buf),
			IoSlice::new(bytes_1),
			IoSlice::new(bytes_2),
		])
	}
}
