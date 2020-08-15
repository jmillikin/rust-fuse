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

use core::mem::{align_of, size_of};
use core::pin::Pin;
use std::ffi::CStr;

use crate::error::{Error, ErrorCode};
use crate::internal::fuse_kernel;

#[cfg(test)]
#[path = "fuse_io_test.rs"]
mod fuse_io_test;

pub(crate) use crate::channel::Channel;

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
	size: usize,
}

impl AlignedVec {
	pub(crate) fn new(size: usize) -> Self {
		let mut vec = Vec::with_capacity(size + 7);
		vec.resize(size + 7, 0u8);
		let pinned = Pin::new(vec.into_boxed_slice());
		let offset = pinned.as_ptr().align_offset(align_of::<u64>());
		Self {
			pinned,
			offset,
			size,
		}
	}
}

impl AlignedBuffer for AlignedVec {
	fn get(&self) -> &[u8] {
		if self.offset == 0 {
			let (aligned, _) = self.pinned.split_at(self.size);
			return aligned;
		}
		let (_, aligned) = self.pinned.split_at(self.offset);
		aligned
	}

	fn get_mut(&mut self) -> &mut [u8] {
		if self.offset == 0 {
			let (aligned, _) = self.pinned.split_at_mut(self.size);
			return aligned;
		}
		let (_, aligned) = self.pinned.split_at_mut(self.offset);
		aligned
	}
}

pub(crate) struct NulTerminatedBytes<'a>(&'a [u8]);

impl<'a> NulTerminatedBytes<'a> {
	pub(crate) fn to_bytes_without_nul(self) -> &'a [u8] {
		&self.0[0..self.0.len() - 1]
	}
}

pub(crate) trait DecodeRequest<'a>: Sized {
	fn decode_request(decoder: RequestDecoder<'a>) -> Result<Self, Error>;
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
	) -> Result<Self, Error> {
		let buf = buf.get();
		if buf.len() < size_of::<fuse_kernel::fuse_in_header>() {
			return Err(Error::UnexpectedEof);
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
			return Err(Error::UnexpectedEof);
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

	fn consume(&self, len: u32) -> Result<u32, Error> {
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
			return Err(Error::UnexpectedEof);
		}
		debug_assert!(new_consumed <= self.header.len);
		Ok(new_consumed)
	}

	pub(crate) fn peek_sized<T: Sized>(&self) -> Result<&'a T, Error> {
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

	pub(crate) fn next_sized<T: Sized>(&mut self) -> Result<&'a T, Error> {
		let out = self.peek_sized()?;
		self.consumed = self.consume(size_of::<T>() as u32)?;
		Ok(out)
	}

	pub(crate) fn next_bytes(&mut self, len: u32) -> Result<&'a [u8], Error> {
		let new_consumed = self.consume(len)?;
		let (_, start) = self.buf.split_at(self.consumed as usize);
		let (out, _) = start.split_at(len as usize);
		self.consumed = new_consumed;
		Ok(out)
	}

	pub(crate) fn next_nul_terminated_bytes(
		&mut self,
	) -> Result<NulTerminatedBytes<'a>, Error> {
		for off in self.consumed..self.header.len {
			if self.buf[off as usize] == 0 {
				let len = off - self.consumed;
				if len == 0 {
					return Err(Error::UnexpectedEof);
				}
				let buf = self.next_bytes(len + 1)?;
				return Ok(NulTerminatedBytes(buf));
			}
		}
		Err(Error::UnexpectedEof)
	}

	pub(crate) fn next_cstr(&mut self) -> Result<&'a CStr, Error> {
		for off in self.consumed..self.header.len {
			if self.buf[off as usize] == 0 {
				let len = off - self.consumed;
				let buf = self.next_bytes(len + 1)?;
				return Ok(unsafe { CStr::from_bytes_with_nul_unchecked(buf) });
			}
		}
		Err(Error::UnexpectedEof)
	}
}

pub(crate) trait EncodeResponse {
	fn encode_response<Chan: Channel>(
		&self,
		enc: ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error>;
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
	pub(crate) fn encode_error(
		self,
		err: ErrorCode,
	) -> Result<(), Chan::Error> {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: -(i32::from(err)),
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			std::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.channel.send(out_hdr_buf)
	}

	pub(crate) fn encode_sized<T: Sized>(
		self,
		t: &T,
	) -> Result<(), Chan::Error> {
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
	) -> Result<(), Chan::Error> {
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
	) -> Result<(), Chan::Error> {
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

	pub(crate) fn encode_header_only(self) -> Result<(), Chan::Error> {
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

		self.channel.send(out_hdr_buf)
	}

	pub(crate) fn encode_bytes(self, bytes: &[u8]) -> Result<(), Chan::Error> {
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
		self.channel.send_vectored(&[out_hdr_buf, bytes])
	}

	pub(crate) fn encode_bytes_2(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
	) -> Result<(), Chan::Error> {
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
		self.channel.send_vectored(&[out_hdr_buf, bytes_1, bytes_2])
	}
}
