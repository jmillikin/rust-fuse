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

use crate::error::ErrorCode;
use crate::internal::fuse_kernel;
use crate::io::{OutputStream, ProtocolVersion};

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
			_phantom: core::marker::PhantomData,
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
