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

//! Request parsing and validation.

use core::ffi::CStr;
use core::marker::PhantomData;
use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::{NodeName, NodeNameError};
use crate::internal::timestamp;
use crate::kernel;
use crate::server::RequestError;

#[cfg(rust_fuse_test = "decode_test")]
#[path = "decode_test.rs"]
mod decode_test;

#[repr(C)]
union RequestBuf<'a> {
	slice: Slice<'a>,
	header: &'a crate::RequestHeader,
	raw_header: &'a kernel::fuse_in_header,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Slice<'a> {
	ptr: *const u8,
	len: usize,
	_phantom: PhantomData<&'a [u8]>,
}

impl<'a> RequestBuf<'a> {
	#[inline]
	#[must_use]
	fn as_slice(&self) -> &'a [u8] {
		unsafe { from_raw_parts(self.slice.ptr, self.slice.len) }
	}
}

pub(crate) struct RequestDecoder<'a> {
	buf: RequestBuf<'a>,
	header_len: usize,
	consumed: usize,
}

impl<'a> RequestDecoder<'a> {
	pub(crate) unsafe fn new_unchecked(buf: &'a [u8]) -> RequestDecoder<'a> {
		let buf = RequestBuf {
			slice: Slice {
				ptr: buf.as_ptr(),
				len: buf.len(),
				_phantom: PhantomData,
			},
		};
		let header_len = buf.raw_header.len as usize;
		Self {
			buf,
			header_len,
			consumed: size_of::<kernel::fuse_in_header>(),
		}
	}

	pub(crate) fn expect_opcode(
		&self,
		opcode: kernel::fuse_opcode,
	) -> Result<(), RequestError> {
		if self.header().opcode != opcode {
			return Err(RequestError::OpcodeMismatch);
		}
		Ok(())
	}

	pub(crate) fn header(&self) -> &'a kernel::fuse_in_header {
		unsafe { self.buf.raw_header }
	}

	#[inline]
	fn consume(&self, len: usize) -> Result<usize, RequestError> {
		match self.consumed.checked_add(len) {
			Some(new_consumed) => {
				if new_consumed > self.header_len {
					return Err(RequestError::UnexpectedEof);
				}
				Ok(new_consumed)
			},
			None => Err(RequestError::UnexpectedEof),
		}
	}

	pub(crate) fn peek_sized<T: Sized>(&self) -> Result<&'a T, RequestError> {
		self.consume(size_of::<T>())?;
		let out: &'a T = unsafe {
			let out_p = self.buf.slice.ptr.add(self.consumed);
			&*(out_p.cast::<T>())
		};
		Ok(out)
	}

	#[inline]
	pub(crate) fn next_sized<T: Sized>(
		&mut self,
	) -> Result<&'a T, RequestError> {
		let next_consumed = self.consume(size_of::<T>())?;
		let out: &'a T = unsafe {
			let out_p = self.buf.slice.ptr.add(self.consumed);
			&*(out_p.cast::<T>())
		};
		self.consumed = next_consumed;
		Ok(out)
	}

	pub(crate) fn next_bytes(
		&mut self,
		len: u32,
	) -> Result<&'a [u8], RequestError> {
		let len = len as usize;
		let new_consumed = self.consume(len)?;
		let out = unsafe {
			let out_p = self.buf.slice.ptr.add(self.consumed);
			from_raw_parts(out_p, len)
		};
		self.consumed = new_consumed;
		Ok(out)
	}

	pub(crate) fn next_node_name(
		&mut self,
	) -> Result<&'a NodeName, RequestError> {
		let buf = self.buf.as_slice();
		for off in self.consumed..self.header_len {
			let c: u8 = unsafe { *buf.get_unchecked(off) };
			if c == b'/' {
				return Err(NodeNameError::ContainsSlash)?;
			}
			if c != 0 {
				continue
			}
			if self.consumed == off {
				return Err(NodeNameError::Empty)?;
			}
			let name = unsafe {
				NodeName::from_bytes_unchecked(
					buf.get_unchecked(self.consumed..off),
				)
			};
			self.consumed = off + 1;
			return Ok(name);
		}
		Err(RequestError::UnexpectedEof)
	}

	pub(crate) fn next_cstr(&mut self) -> Result<&'a CStr, RequestError> {
		let buf = self.buf.as_slice();
		for off in self.consumed..self.header_len {
			let c: u8 = unsafe { *buf.get_unchecked(off) };
			if c != 0 {
				continue
			}
			if self.consumed == off {
				return Ok(c"");
			}
			let new_consumed = off + 1;
			let cstr = unsafe {
				CStr::from_bytes_with_nul_unchecked(
					buf.get_unchecked(self.consumed..new_consumed),
				)
			};
			self.consumed = new_consumed;
			return Ok(cstr);
		}
		Err(RequestError::UnexpectedEof)
	}
}

pub(crate) fn node_id(raw: u64) -> Result<crate::NodeId, RequestError> {
	match crate::NodeId::new(raw) {
		Some(x) => Ok(x),
		None => Err(RequestError::MissingNodeId),
	}
}

pub(crate) fn check_timespec_nanos(nanos: u32) -> Result<(), RequestError> {
	if nanos > timestamp::MAX_NANOS {
		return Err(RequestError::TimestampOverflow);
	}
	Ok(())
}
