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
use crate::server;
use crate::server::RequestError;

#[cfg(rust_fuse_test = "decode_test")]
#[path = "decode_test.rs"]
mod decode_test;

mod sealed {
	pub trait Sealed: Sized {}
}

pub(crate) use sealed::Sealed;

/// A trait for request types that are valid for CUSE servers.
pub trait CuseRequest<'a>: Sealed {
	fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, RequestError>;
}

/// A trait for request types that are valid for FUSE servers.
pub trait FuseRequest<'a>: Sealed {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, RequestError>;
}

#[derive(Copy, Clone)]
pub(crate) union RequestBuf<'a> {
	buf: Slice<'a>,
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
	pub(crate) fn new(
		buf: crate::io::AlignedSlice<'a>,
	) -> Result<Self, RequestError> {
		let buf = buf.get();
		if buf.len() < size_of::<fuse_kernel::fuse_in_header>() {
			return Err(RequestError::UnexpectedEof);
		}

		let header_ptr = buf.as_ptr().cast::<fuse_kernel::fuse_in_header>();
		let header = unsafe { &*header_ptr };

		if header.unique == 0 {
			return Err(RequestError::MissingRequestId);
		}

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

	pub(crate) fn header(self) -> &'a crate::RequestHeader {
		unsafe { self.header }
	}

	pub(crate) fn raw_header(self) -> &'a fuse_kernel::fuse_in_header {
		unsafe { self.raw_header }
	}

	pub(crate) fn expect_opcode(
		&self,
		opcode: fuse_kernel::fuse_opcode,
	) -> Result<(), RequestError> {
		if self.raw_header().opcode != opcode {
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
		self.buf.raw_header()
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
