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

//! I/O helpers common to both clients and servers.

use core::mem;

mod buffer;
pub(crate) use self::buffer::ArrayBuffer;

/// A buffer of sendable data split into chunks.
///
/// A `SendBuf<'a>` is equivalent to `[&'a [u8]; N]`, where `N` is a small
/// constant guaranteed to be not more than [`MAX_CHUNKS_LEN`]. This allows
/// converting a `SendBuf` to an array of [`IoSlice`] (or equivalent) without
/// performing dynamic allocation.
///
/// [`MAX_CHUNKS_LEN`]: SendBuf::MAX_CHUNKS_LEN
/// [`IoSlice`]: std::io::IoSlice
pub struct SendBuf<'a> {
	len: usize,
	chunks: [&'a [u8]; SendBuf::MAX_CHUNKS_LEN],
	chunks_len: usize,
}

impl SendBuf<'_> {
	/// The maximum number of chunks contained within a `SendBuf`.
	///
	/// This constant is intended to be opaque, and its exact value should not
	/// be hardcoded into calling code.
	pub const MAX_CHUNKS_LEN: usize = 5;
}

impl<'a> SendBuf<'a> {
	#[inline]
	#[must_use]
	pub(crate) fn new_1(len: usize, chunk_1: &'a [u8]) -> Self {
		Self {
			len,
			chunks: [chunk_1, b"", b"", b"", b""],
			chunks_len: 1,
		}
	}

	#[inline]
	#[must_use]
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
	#[must_use]
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
	#[must_use]
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
	#[must_use]
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
	/// Returns the chunks contained within this `SendBuf` as a slice.
	///
	/// The slice length will not exceed [`MAX_CHUNKS_LEN`].
	///
	/// [`MAX_CHUNKS_LEN`]: SendBuf::MAX_CHUNKS_LEN
	#[inline]
	#[must_use]
	pub fn chunks(&self) -> &[&'a [u8]] {
		&self.chunks[..self.chunks_len()]
	}

	/// Returns the number of chunks within this `SendBuf`.
	///
	/// The returned value will not exceed [`MAX_CHUNKS_LEN`].
	///
	/// [`MAX_CHUNKS_LEN`]: SendBuf::MAX_CHUNKS_LEN
	#[inline]
	#[must_use]
	pub fn chunks_len(&self) -> usize {
		if self.chunks_len <= SendBuf::MAX_CHUNKS_LEN {
			return self.chunks_len;
		}
		unsafe { std::hint::unreachable_unchecked() }
	}

	/// Returns the number of bytes within this `SendBuf`.
	#[inline]
	#[must_use]
	pub fn len(&self) -> usize {
		self.len
	}

	/// Returns the bytes within this `SendBuf` as a dynamically-allocated
	/// `Vec<u8>`.
	///
	/// # Errors
	///
	/// Returns an error on capacity overflow or allocation failure.
	#[cfg(any(doc, feature = "std"))]
	#[inline]
	pub fn to_vec(&self) -> Result<Vec<u8>, std::collections::TryReserveError> {
		let mut vec = Vec::new();
		vec.try_reserve_exact(self.len)?;
		for chunk in self.chunks() {
			vec.extend_from_slice(chunk);
		}
		Ok(vec)
	}

	/// Call a closure on each chunk, placing the results into provided storage.
	///
	/// The returned slice contains the portion of the storage that was
	/// filled in by this function. The rest of the storage will be left
	/// unchanged.
	///
	/// # Examples
	///
	/// ```rust
	/// use std::io::IoSlice;
	/// use fuse::io::SendBuf;
	///
	/// fn send(buf: &SendBuf) {
	/// 	let mut storage = [IoSlice::new(b""); SendBuf::MAX_CHUNKS_LEN];
	/// 	let io_slices = buf.map_chunks_into(&mut storage, |chunk| {
	/// 		IoSlice::new(chunk)
	/// 	});
	///
	/// 	assert_eq!(io_slices.len(), buf.chunks_len());
	/// }
	/// ```
	#[inline]
	#[must_use]
	pub fn map_chunks_into<'b, T>(
		&self,
		storage: &'b mut [T; SendBuf::MAX_CHUNKS_LEN],
		mut f: impl FnMut(&'a [u8]) -> T,
	) -> &'b [T] {
		for (ii, chunk) in self.chunks().iter().enumerate() {
			storage[ii] = f(chunk);
		}
		&storage[..self.chunks_len()]
	}

	/// Call a closure on each chunk, placing the results into uninitialized
	/// storage.
	///
	/// The returned slice contains the portion of the storage that was
	/// initialized by this function. The rest of the storage will be left
	/// unchanged, and may contain uninitialized values.
	///
	/// See documentation for [`core::mem::MaybeUninit`] regarding how to
	/// safely work with uninitialized values and partially-initialized arrays.
	///
	/// # Examples
	///
	/// ```rust
	/// use std::io::IoSlice;
	/// use std::mem::MaybeUninit;
	/// use fuse::io::SendBuf;
	///
	/// fn send(buf: &SendBuf) {
	/// 	type UninitSlice<'a> = MaybeUninit<IoSlice<'a>>;
	/// 	let mut storage: [UninitSlice; SendBuf::MAX_CHUNKS_LEN] = unsafe {
	/// 		MaybeUninit::uninit().assume_init()
	/// 	};
	/// 	let io_slices = buf.map_chunks_into_uninit(&mut storage, |chunk| {
	/// 		IoSlice::new(chunk)
	/// 	});
	///
	/// 	assert_eq!(io_slices.len(), buf.chunks_len());
	/// }
	/// ```
	#[inline]
	#[must_use]
	pub fn map_chunks_into_uninit<'b, T>(
		&self,
		storage: &'b mut [mem::MaybeUninit<T>; SendBuf::MAX_CHUNKS_LEN],
		mut f: impl FnMut(&'a [u8]) -> T,
	) -> &'b [T] {
		for (ii, chunk) in self.chunks().iter().enumerate() {
			storage[ii] = mem::MaybeUninit::new(f(chunk));
		}
		let out = &storage[..self.chunks_len()];
		unsafe {
			&*(out as *const [mem::MaybeUninit<T>] as *const [T])
		}
	}
}
