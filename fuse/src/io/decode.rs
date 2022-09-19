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

use core::marker::PhantomData;
use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::internal::fuse_kernel;
use crate::io::Buffer;

#[cfg(rust_fuse_test = "decode_test")]
#[path = "decode_test.rs"]
mod decode_test;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReplyError {
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestError {
	InvalidLockType,
	MissingNodeId,
	OpcodeMismatch,
	UnexpectedEof,
}

#[derive(Copy, Clone)]
pub(crate) union RequestBuf<'a> {
	buf: Slice<'a>,
	header: &'a fuse_kernel::fuse_in_header,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Slice<'a> {
	ptr: *const u8,
	len: usize,
	_phantom: PhantomData<&'a [u8]>,
}

impl<'a> RequestBuf<'a> {
	pub(crate) fn new(
		buf: &'a impl Buffer,
		recv_len: usize,
	) -> Result<Self, RequestError> {
		// TODO: validate recv_len
		if recv_len < size_of::<fuse_kernel::fuse_in_header>() {
			return Err(RequestError::UnexpectedEof);
		}
		let buf = &buf.borrow()[..recv_len];
		if buf.len() < size_of::<fuse_kernel::fuse_in_header>() {
			return Err(RequestError::UnexpectedEof);
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
			return Err(RequestError::UnexpectedEof);
		}

		Ok(RequestBuf {
			buf: Slice {
				ptr: buf.as_ptr(),
				len: buf.len(),
				_phantom: PhantomData,
			},
		})
	}

	pub(crate) fn header(self) -> &'a fuse_kernel::fuse_in_header {
		unsafe { self.header }
	}

	pub(crate) fn expect_opcode(
		&self,
		opcode: fuse_kernel::fuse_opcode,
	) -> Result<(), RequestError> {
		if self.header().opcode != opcode {
			return Err(RequestError::OpcodeMismatch);
		}
		Ok(())
	}

	fn as_ptr(&self) -> *const u8 {
		unsafe { self.buf.ptr }
	}

	fn as_slice(&self) -> &'a [u8] {
		unsafe { from_raw_parts(self.buf.ptr, self.buf.len) }
	}
}

pub(crate) struct RequestDecoder<'a> {
	buf: RequestBuf<'a>,
	consumed: u32,
}

impl<'a> RequestDecoder<'a> {
	pub(crate) fn new(buf: RequestBuf<'a>) -> Self {
		RequestDecoder {
			buf,
			consumed: size_of::<fuse_kernel::fuse_in_header>() as u32,
		}
	}

	pub(crate) fn expect_opcode(
		&self,
		opcode: fuse_kernel::fuse_opcode,
	) -> Result<(), RequestError> {
		self.buf.expect_opcode(opcode)
	}

	pub(crate) fn header(&self) -> &'a fuse_kernel::fuse_in_header {
		self.buf.header()
	}

	fn consume(&self, len: u32) -> Result<u32, RequestError> {
		let new_consumed: u32;
		let eof: bool;
		match self.consumed.checked_add(len) {
			Some(x) => {
				new_consumed = x;
				eof = new_consumed > self.header().len;
			},
			None => {
				new_consumed = 0;
				eof = true;
			},
		}
		if eof {
			return Err(RequestError::UnexpectedEof);
		}
		debug_assert!(new_consumed <= self.header().len);
		Ok(new_consumed)
	}

	pub(crate) fn peek_sized<T: Sized>(&self) -> Result<&'a T, RequestError> {
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

	pub(crate) fn next_sized<T: Sized>(
		&mut self,
	) -> Result<&'a T, RequestError> {
		let out = self.peek_sized()?;
		self.consumed = self.consume(size_of::<T>() as u32)?;
		Ok(out)
	}

	pub(crate) fn next_bytes(
		&mut self,
		len: u32,
	) -> Result<&'a [u8], RequestError> {
		let new_consumed = self.consume(len)?;
		let (_, start) = self.buf.as_slice().split_at(self.consumed as usize);
		let (out, _) = start.split_at(len as usize);
		self.consumed = new_consumed;
		Ok(out)
	}

	pub(crate) fn next_nul_terminated_bytes(
		&mut self,
	) -> Result<NulTerminatedBytes<'a>, RequestError> {
		for off in self.consumed..self.header().len {
			if self.buf.as_slice()[off as usize] == 0 {
				let len = off - self.consumed;
				if len == 0 {
					return Err(RequestError::UnexpectedEof);
				}
				let buf = self.next_bytes(len + 1)?;
				return Ok(NulTerminatedBytes(buf));
			}
		}
		Err(RequestError::UnexpectedEof)
	}
}

pub(crate) struct NulTerminatedBytes<'a>(&'a [u8]);

impl<'a> NulTerminatedBytes<'a> {
	pub(crate) fn to_bytes_without_nul(self) -> &'a [u8] {
		&self.0[0..self.0.len() - 1]
	}
}
