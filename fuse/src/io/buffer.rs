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

use core::borrow::{Borrow, BorrowMut};
#[cfg(feature = "std")]
use core::mem::align_of;
#[cfg(feature = "std")]
use core::pin::Pin;
#[cfg(feature = "std")]
use core::slice;

use crate::internal::fuse_kernel;

pub unsafe trait Buffer {
	fn borrow(&self) -> &[u8];
	fn borrow_mut(&mut self) -> &mut [u8];
}

#[derive(Copy, Clone)]
pub struct AlignedSlice<'a>(&'a [u8]);

impl<'a> AlignedSlice<'a> {
	pub(crate) fn new(buf: &'a impl Buffer) -> Self {
		Self(buf.borrow())
	}

	pub(crate) fn get(&self) -> &'a [u8] { self.0 }
}

pub struct ArrayBuffer(ArrayBufferImpl);

#[repr(align(8))]
struct ArrayBufferImpl([u8; MIN_READ_BUFFER]);

impl ArrayBuffer {
	pub fn new() -> Self {
		ArrayBuffer(ArrayBufferImpl([0u8; MIN_READ_BUFFER]))
	}
}

pub const MIN_READ_BUFFER: usize = fuse_kernel::FUSE_MIN_READ_BUFFER;

unsafe impl Buffer for ArrayBuffer {
	fn borrow(&self) -> &[u8] {
		&self.0 .0
	}
	fn borrow_mut(&mut self) -> &mut [u8] {
		&mut self.0 .0
	}
}

impl Borrow<[u8]> for ArrayBuffer {
	fn borrow(&self) -> &[u8] {
		Buffer::borrow(self)
	}
}

impl BorrowMut<[u8]> for ArrayBuffer {
	fn borrow_mut(&mut self) -> &mut [u8] {
		Buffer::borrow_mut(self)
	}
}

#[cfg(feature = "std")]
pub struct PinnedBuffer {
	_pinned: Pin<Box<[u8]>>,
	ptr: *mut u8,
	size: usize,
}

#[cfg(feature = "std")]
impl PinnedBuffer {
	pub fn new(size: usize) -> Self {
		let size = core::cmp::min(size, MIN_READ_BUFFER);
		let mut vec = Vec::with_capacity(size + 7);
		vec.resize(size + 7, 0u8);
		let mut pinned = Pin::new(vec.into_boxed_slice());
		let ptr = pinned.as_mut_ptr();
		let offset = ptr.align_offset(align_of::<u64>());
		Self {
			_pinned: pinned,
			ptr: unsafe { ptr.add(offset) },
			size,
		}
	}
}

#[cfg(feature = "std")]
unsafe impl Buffer for PinnedBuffer {
	fn borrow(&self) -> &[u8] {
		unsafe { slice::from_raw_parts(self.ptr, self.size) }
	}

	fn borrow_mut(&mut self) -> &mut [u8] {
		unsafe { slice::from_raw_parts_mut(self.ptr, self.size) }
	}
}

#[cfg(feature = "std")]
impl Borrow<[u8]> for PinnedBuffer {
	fn borrow(&self) -> &[u8] {
		Buffer::borrow(self)
	}
}

#[cfg(feature = "std")]
impl BorrowMut<[u8]> for PinnedBuffer {
	fn borrow_mut(&mut self) -> &mut [u8] {
		Buffer::borrow_mut(self)
	}
}
