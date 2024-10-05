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

use crate::internal::fuse_kernel;

/// The minimum buffer size (in bytes) for receiving a FUSE message.
///
/// The FUSE client may reject reads into buffers smaller than
/// `FUSE_MIN_READ_BUFFER` bytes.
///
/// Note that the minimum buffer size is *not* the maximum size of a FUSE
/// message:
///
/// * The server is allowed to negotiate a larger `max_write` than the default
///   of `4096`, in which case the minimum size of a `FUSE_WRITE` message will
///   be `max_write` + `size_of::<fuse_in_header>` + `size_of::<fuse_write_in>`.
///
///   In this case the client may reject reads into buffers that aren't large
///   enough to store the largest possible `FUSE_WRITE` message.
///
/// * A `FUSE_SETXATTR` message may contain values up to 64 KiB (on Linux), or
///   even of unlimited size (on FreeBSD). See [`crate::XattrValue::MAX_LEN`] for
///   details.
///
/// [`crate::XattrValue::MAX_LEN`]: crate::XattrValue::MAX_LEN
pub const FUSE_MIN_READ_BUFFER: usize = fuse_kernel::FUSE_MIN_READ_BUFFER;

// AlignedSlice {{{

/// A byte slice correctly aligned for decoding FUSE messages.
///
/// An instance of this type is a static guarantee that the underlying byte
/// slice is sufficiently aligned to store an encoded FUSE message.
#[derive(Clone, Copy, Debug)]
pub struct AlignedSlice<'a> {
	inner: &'a [u8],
}

/// Values that can be borrowed as aligned slices.
pub trait AsAlignedSlice {
	/// Borrow the value as an aligned slice.
	fn as_aligned_slice(&self) -> AlignedSlice;
}

impl<'a> AlignedSlice<'a> {
	/// Creates an `AlignedSlice` from a byte slice, if properly aligned.
	#[inline]
	#[must_use]
	pub fn new(slice: &'a [u8]) -> Option<AlignedSlice<'a>> {
		let offset = slice.as_ptr().align_offset(mem::align_of::<u64>());
		if offset == 0 {
			return Some(Self { inner: slice });
		}
		None
	}

	/// Creates an `AlignedSlice` from a byte slice without validating
	/// alignment.
	///
	/// # Safety
	///
	/// The provided slice must be aligned to an 8-byte boundary.
	#[inline]
	#[must_use]
	pub unsafe fn new_unchecked(slice: &'a [u8]) -> AlignedSlice<'a> {
		Self { inner: slice }
	}

	/// Returns the underlying byte slice.
	#[inline]
	#[must_use]
	pub fn get(&self) -> &'a [u8] {
		self.inner
	}

	/// Truncates the `AlignedSlice` to be no longer than the given length.
	///
	/// If `len` is greater than the slice's current length, the original
	/// `AlignedSlice` is returned unchanged.
	#[inline]
	#[must_use]
	pub fn truncate(self, len: usize) -> AlignedSlice<'a> {
		if len >= self.inner.len() {
			return self;
		}
		Self {
			inner: &self.inner[..len],
		}
	}
}

// }}}

// AlignedSliceMut {{{

/// A mutable byte slice correctly aligned for decoding FUSE messages.
///
/// An instance of this type is a static guarantee that the underlying byte
/// slice is sufficiently aligned to store an encoded FUSE message.
#[derive(Debug)]
pub struct AlignedSliceMut<'a> {
	inner: &'a mut [u8],
}

/// Values that can be borrowed as mutable aligned slices.
pub trait AsAlignedSliceMut: AsAlignedSlice {
	/// Borrow the value as a mutable aligned slice.
	fn as_aligned_slice_mut(&mut self) -> AlignedSliceMut;
}

impl<'a> AlignedSliceMut<'a> {
	/// Creates an `AlignedSliceMut` from a byte slice, if properly aligned.
	#[inline]
	#[must_use]
	pub fn new(slice_mut: &'a mut [u8]) -> Option<AlignedSliceMut<'a>> {
		let offset = slice_mut.as_ptr().align_offset(mem::align_of::<u64>());
		if offset == 0 {
			return Some(Self { inner: slice_mut });
		}
		None
	}

	/// Creates an `AlignedSliceMut` from a byte slice without validating
	/// alignment.
	///
	/// # Safety
	///
	/// The provided slice must be aligned to an 8-byte boundary.
	#[inline]
	#[must_use]
	pub unsafe fn new_unchecked(
		slice_mut: &'a mut [u8],
	) -> AlignedSliceMut<'a> {
		Self { inner: slice_mut }
	}

	/// Returns the underlying byte slice.
	#[inline]
	#[must_use]
	pub fn get_mut(&mut self) -> &mut [u8] {
		self.inner
	}

	/// Truncates the `AlignedSliceMut` to be no longer than the given length.
	///
	/// If `len` is greater than the slice's current length, the original
	/// `AlignedSliceMut` is returned unchanged.
	#[inline]
	#[must_use]
	pub fn truncate(self, len: usize) -> AlignedSliceMut<'a> {
		if len >= self.inner.len() {
			return self;
		}
		Self {
			inner: &mut self.inner[..len],
		}
	}
}

impl<'a> From<AlignedSliceMut<'a>> for AlignedSlice<'a> {
	fn from(other: AlignedSliceMut<'a>) -> AlignedSlice<'a> {
		Self { inner: other.inner }
	}
}

// }}}

// MinReadBuffer {{{

/// A fixed-size aligned buffer of [`FUSE_MIN_READ_BUFFER`] bytes.
#[derive(Clone, Copy, Debug)]
#[repr(align(8))]
pub struct MinReadBuffer {
	bytes: [u8; MinReadBuffer::LEN],
}

impl MinReadBuffer {
	/// The length of a `MinReadBuffer`.
	///
	/// This is a convenience alias for [`FUSE_MIN_READ_BUFFER`].
	pub const LEN: usize = FUSE_MIN_READ_BUFFER;

	/// Creates a new `MinReadBuffer` initialized with zeros.
	#[inline]
	#[must_use]
	pub const fn new() -> MinReadBuffer {
		Self {
			bytes: [0u8; MinReadBuffer::LEN],
		}
	}

	/// Borrows the buffer as a byte slice.
	#[inline]
	#[must_use]
	pub fn as_slice(&self) -> &[u8; MinReadBuffer::LEN] {
		&self.bytes
	}

	/// Borrows the buffer as a mutable byte slice.
	#[inline]
	#[must_use]
	pub fn as_slice_mut(&mut self) -> &mut [u8; MinReadBuffer::LEN] {
		&mut self.bytes
	}

	/// Borrows the buffer as an aligned byte slice.
	#[inline]
	#[must_use]
	pub fn as_aligned_slice(&self) -> AlignedSlice {
		AlignedSlice { inner: &self.bytes }
	}

	/// Borrows the buffer as a mutable aligned byte slice.
	#[inline]
	#[must_use]
	pub fn as_aligned_slice_mut(&mut self) -> AlignedSliceMut {
		AlignedSliceMut { inner: &mut self.bytes }
	}
}

impl AsAlignedSlice for MinReadBuffer {
	fn as_aligned_slice(&self) -> AlignedSlice {
		self.as_aligned_slice()
	}
}

impl AsAlignedSliceMut for MinReadBuffer {
	fn as_aligned_slice_mut(&mut self) -> AlignedSliceMut {
		self.as_aligned_slice_mut()
	}
}

// }}}

// SendBuf {{{

/// A buffer of sendable data split into chunks.
///
/// A `SendBuf<'a>` is equivalent to `[&'a [u8]; N]`, where `N` is a small
/// constant guaranteed to be not more than [`MAX_CHUNKS_LEN`]. This allows
/// converting a `SendBuf` to an array of [`IoSlice`] (or equivalent) without
/// performing dynamic allocation.
///
/// [`MAX_CHUNKS_LEN`]: SendBuf::MAX_CHUNKS_LEN
/// [`IoSlice`]: https://doc.rust-lang.org/std/io/struct.IoSlice.html
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
		unsafe { core::hint::unreachable_unchecked() }
	}

	/// Returns the number of bytes within this `SendBuf`.
	#[inline]
	#[must_use]
	pub fn len(&self) -> usize {
		self.len
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

// }}}
