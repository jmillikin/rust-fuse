// Copyright 2021 John Millikin and the rust-fuse contributors.
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

use core::mem;

mod buffer;
pub(crate) use self::buffer::ArrayBuffer;

pub struct SendBuf<'a> {
	len: usize,
	chunks: [&'a [u8]; SEND_BUF_MAX_CHUNKS_LEN],
	chunks_len: usize,
}

impl SendBuf<'_> {
	pub const MAX_CHUNKS_LEN: usize = SEND_BUF_MAX_CHUNKS_LEN;
}

const SEND_BUF_MAX_CHUNKS_LEN: usize = 5;

impl<'a> SendBuf<'a> {
	#[inline]
	pub(crate) fn new_1(len: usize, chunk_1: &'a [u8]) -> Self {
		Self {
			len,
			chunks: [chunk_1, b"", b"", b"", b""],
			chunks_len: 1,
		}
	}

	#[inline]
	pub(crate) fn new_2(
		len: usize,
		chunk_1: &'a [u8],
		chunk_2: &'a [u8],
	) -> Self {
		Self {
			len,
			chunks: [chunk_1, chunk_2, b"", b"", b""],
			chunks_len: 2,
		}
	}

	#[inline]
	pub(crate) fn new_3(
		len: usize,
		chunk_1: &'a [u8],
		chunk_2: &'a [u8],
		chunk_3: &'a [u8],
	) -> Self {
		Self {
			len,
			chunks: [chunk_1, chunk_2, chunk_3, b"", b""],
			chunks_len: 3,
		}
	}

	#[inline]
	pub(crate) fn new_4(
		len: usize,
		chunk_1: &'a [u8],
		chunk_2: &'a [u8],
		chunk_3: &'a [u8],
		chunk_4: &'a [u8],
	) -> Self {
		Self {
			len,
			chunks: [chunk_1, chunk_2, chunk_3, chunk_4, b""],
			chunks_len: 4,
		}
	}

	#[inline]
	pub(crate) fn new_5(
		len: usize,
		chunk_1: &'a [u8],
		chunk_2: &'a [u8],
		chunk_3: &'a [u8],
		chunk_4: &'a [u8],
		chunk_5: &'a [u8],
	) -> Self {
		Self {
			len,
			chunks: [chunk_1, chunk_2, chunk_3, chunk_4, chunk_5],
			chunks_len: 5,
		}
	}
}

impl<'a> SendBuf<'a> {
	#[inline]
	#[must_use]
	pub fn chunks(&self) -> &[&'a [u8]] {
		unsafe { self.chunks.get_unchecked(..self.chunks_len) }
	}

	#[inline]
	#[must_use]
	pub fn chunks_len(&self) -> usize {
		self.chunks_len
	}

	#[inline]
	#[must_use]
	pub fn len(&self) -> usize {
		self.len
	}

	#[cfg(any(doc, feature = "std"))]
	#[must_use]
	pub fn to_vec(&self) -> Vec<u8> {
		let mut vec = Vec::with_capacity(self.len);
		for chunk in self.chunks() {
			vec.extend_from_slice(chunk);
		}
		vec
	}

	#[inline]
	#[must_use]
	pub fn map_chunks_into<'b, T>(
		&self,
		capacity: &'b mut [T; SendBuf::MAX_CHUNKS_LEN],
		mut f: impl FnMut(&'a [u8]) -> T,
	) -> &'b [T] {
		unsafe {
			#[allow(clippy::needless_range_loop)]
			for ii in 0..self.chunks_len {
				let buf = self.chunks.get_unchecked(ii);
				capacity[ii] = f(buf);
			}
			capacity.get_unchecked(..self.chunks_len)
		}
	}

	#[inline]
	#[must_use]
	pub fn map_chunks_into_uninit<'b, T>(
		&self,
		capacity: &'b mut [mem::MaybeUninit<T>; SendBuf::MAX_CHUNKS_LEN],
		mut f: impl FnMut(&'a [u8]) -> T,
	) -> &'b [T] {
		unsafe {
			#[allow(clippy::needless_range_loop)]
			for ii in 0..self.chunks_len {
				let buf = self.chunks.get_unchecked(ii);
				capacity[ii] = mem::MaybeUninit::new(f(buf));
			}
			let out = capacity.get_unchecked(..self.chunks_len);
			&*(out as *const [mem::MaybeUninit<T>] as *const [T])
		}
	}
}
