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

use core::marker::PhantomData;
use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::internal::fuse_kernel;
use crate::internal::timestamp;
use crate::node;
use crate::server::RequestError;

#[cfg(rust_fuse_test = "decode_test")]
#[path = "decode_test.rs"]
mod decode_test;

#[repr(C)]
union RequestBuf<'a> {
	slice: Slice<'a>,
	header: &'a crate::RequestHeader,
	raw_header: &'a fuse_kernel::fuse_in_header,
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
	consumed: u32,
}

impl<'a> RequestDecoder<'a> {
	pub(crate) unsafe fn new_unchecked(buf: &'a [u8]) -> RequestDecoder<'a> {
		Self {
			buf: RequestBuf {
				slice: Slice {
					ptr: buf.as_ptr(),
					len: buf.len(),
					_phantom: PhantomData,
				},
			},
			consumed: size_of::<fuse_kernel::fuse_in_header>() as u32,
		}
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

	pub(crate) fn header(&self) -> &'a fuse_kernel::fuse_in_header {
		unsafe { self.buf.raw_header }
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
			let out_p = self.buf.slice.ptr.add(self.consumed as usize);
			&*(out_p.cast::<T>())
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

	pub(crate) fn next_node_name(
		&mut self,
	) -> Result<&'a node::Name, RequestError> {
		let bytes = self.next_nul_terminated_bytes()?;
		Ok(node::Name::from_bytes(bytes.to_bytes_without_nul())?)
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
	#[allow(clippy::wrong_self_convention)] // TODO
	pub(crate) fn to_bytes_without_nul(self) -> &'a [u8] {
		&self.0[0..self.0.len() - 1]
	}
}

pub(crate) fn node_id(raw: u64) -> Result<node::Id, RequestError> {
	match node::Id::new(raw) {
		Some(x) => Ok(x),
		None => Err(RequestError::MissingNodeId),
	}
}

pub(crate) fn check_timespec_nanos(nanos: u32) -> Result<(), RequestError> {
	if nanos > timestamp::MAX_NANOS {
		return Err(RequestError::TimestampNanosOverflow);
	}
	Ok(())
}
