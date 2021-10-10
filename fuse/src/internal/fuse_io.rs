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

use core::mem::size_of;

use crate::error::{Error, ErrorCode};
use crate::internal::fuse_kernel;
use crate::io::{Buffer, OutputStream, ProtocolVersion};

#[cfg(rust_fuse_test = "fuse_io_test")]
#[path = "fuse_io_test.rs"]
mod fuse_io_test;

#[allow(dead_code)]
pub(crate) fn aligned_borrow<'a, Buf: Buffer>(
	buf: &'a Buf,
) -> AlignedSlice<'a> {
	AlignedSlice {
		buf: &buf.borrow(),
	}
}

pub(crate) fn aligned_slice<'a, Buf: Buffer>(
	buf: &'a Buf,
	size: usize,
) -> AlignedSlice<'a> {
	// TODO: validate size
	AlignedSlice {
		buf: &buf.borrow()[0..size],
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

pub(crate) struct NulTerminatedBytes<'a>(&'a [u8]);

impl<'a> NulTerminatedBytes<'a> {
	pub(crate) fn to_bytes_without_nul(self) -> &'a [u8] {
		&self.0[0..self.0.len() - 1]
	}
}

pub(crate) trait DecodeRequest<'a>: Sized {
	fn decode_request(decoder: RequestDecoder<'a>) -> Result<Self, Error>;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Semantics {
	FUSE,
	CUSE,
}

pub(crate) struct RequestDecoder<'a> {
	buf: &'a [u8],
	header: &'a fuse_kernel::fuse_in_header,
	version: ProtocolVersion,
	semantics: Semantics,
	consumed: u32,
}

impl<'a> RequestDecoder<'a> {
	pub(crate) fn new(
		buf: AlignedSlice<'a>,
		version: ProtocolVersion,
		semantics: Semantics,
	) -> Result<Self, Error> {
		let buf = buf.get();
		if buf.len() < size_of::<fuse_kernel::fuse_in_header>() {
			return Err(Error::unexpected_eof());
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
			return Err(Error::unexpected_eof());
		}

		Ok(RequestDecoder {
			buf,
			header,
			version,
			semantics,
			consumed: size_of::<fuse_kernel::fuse_in_header>() as u32,
		})
	}

	pub(crate) fn header(&self) -> &'a fuse_kernel::fuse_in_header {
		self.header
	}

	pub(crate) fn version(&self) -> ProtocolVersion {
		self.version
	}

	pub(crate) fn is_cuse(&self) -> bool {
		self.semantics == Semantics::CUSE
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
			return Err(Error::unexpected_eof());
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
					return Err(Error::unexpected_eof());
				}
				let buf = self.next_bytes(len + 1)?;
				return Ok(NulTerminatedBytes(buf));
			}
		}
		Err(Error::unexpected_eof())
	}
}

pub(crate) trait EncodeResponse {
	fn encode_response<S: OutputStream>(
		&self,
		enc: ResponseEncoder<S>,
	) -> Result<(), S::Error>;
}

pub(crate) struct ResponseEncoder<'a, S> {
	stream: S,
	request_id: u64,
	version: ProtocolVersion,
	_phantom: core::marker::PhantomData<&'a S>,
}

impl<'a, S> ResponseEncoder<'a, S> {
	pub(crate) fn new(
		stream: S,
		request_id: u64,
		version: ProtocolVersion,
	) -> Self {
		Self {
			stream,
			request_id,
			version,
			_phantom: core::marker::PhantomData {},
		}
	}

	pub(crate) fn version(&self) -> ProtocolVersion {
		self.version
	}
}

impl<S: OutputStream> ResponseEncoder<'_, S> {
	pub(crate) fn encode_error(self, err: ErrorCode) -> Result<(), S::Error> {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: -(i32::from(err)),
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			core::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.stream.send(out_hdr_buf)
	}

	pub(crate) fn encode_sized<T: Sized>(self, t: &T) -> Result<(), S::Error> {
		let bytes: &[u8] = unsafe {
			core::slice::from_raw_parts(
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
	) -> Result<(), S::Error> {
		let bytes_2: &[u8] = unsafe {
			core::slice::from_raw_parts(
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
	) -> Result<(), S::Error> {
		let bytes_1: &[u8] = unsafe {
			core::slice::from_raw_parts(
				(t_1 as *const T1) as *const u8,
				size_of::<T1>(),
			)
		};
		let bytes_2: &[u8] = unsafe {
			core::slice::from_raw_parts(
				(t_2 as *const T2) as *const u8,
				size_of::<T2>(),
			)
		};
		self.encode_bytes_2(bytes_1, bytes_2)
	}

	pub(crate) fn encode_header_only(self) -> Result<(), S::Error> {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf: &[u8] = unsafe {
			core::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.stream.send(out_hdr_buf)
	}

	pub(crate) fn encode_bytes(self, bytes: &[u8]) -> Result<(), S::Error> {
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
			core::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.stream.send_vectored(&[out_hdr_buf, bytes])
	}

	pub(crate) fn encode_bytes_2(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
	) -> Result<(), S::Error> {
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
			core::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.stream.send_vectored(&[out_hdr_buf, bytes_1, bytes_2])
	}

	pub(crate) fn encode_bytes_4(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
		bytes_3: &[u8],
		bytes_4: &[u8],
	) -> Result<(), S::Error> {
		let mut len = size_of::<fuse_kernel::fuse_out_header>();

		match len.checked_add(bytes_1.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_1.len()),
		}
		match len.checked_add(bytes_2.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_2.len()),
		}
		match len.checked_add(bytes_3.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_3.len()),
		}
		match len.checked_add(bytes_4.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_4.len()),
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
			core::slice::from_raw_parts(
				(&out_hdr as *const fuse_kernel::fuse_out_header) as *const u8,
				size_of::<fuse_kernel::fuse_out_header>(),
			)
		};

		self.stream.send_vectored(&[
			out_hdr_buf,
			bytes_1,
			bytes_2,
			bytes_3,
			bytes_4,
		])
	}
}
