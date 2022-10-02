// Copyright 2022 John Millikin and the rust-fuse contributors.
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
use core::ptr;
use core::slice;

use crate::internal::fuse_kernel;

pub(crate) trait Dirent: Sized {
	fn namelen(&self) -> u32;
}

impl Dirent for fuse_kernel::fuse_dirent {
	#[inline]
	fn namelen(&self) -> u32 {
		self.namelen
	}
}

#[inline]
pub(crate) fn entry_size<T: Dirent>(name: &crate::NodeName) -> usize {
	let name_len = name.as_bytes().len();
	let padding_len = (8 - (name_len % 8)) % 8;
	mem::size_of::<T>() + name_len + padding_len
}

#[inline]
pub(crate) unsafe fn read_unchecked<T: Dirent>(
	buf: &[u8],
) -> (T, &crate::NodeName) {
	let buf_ptr = buf.as_ptr();
	let name_ptr = buf_ptr.add(mem::size_of::<T>());

	let mut dirent_uninit: mem::MaybeUninit<T> = mem::MaybeUninit::uninit();
	ptr::copy_nonoverlapping(
		buf_ptr,
		dirent_uninit.as_mut_ptr() as *mut u8,
		mem::size_of::<T>(),
	);
	let dirent = dirent_uninit.assume_init();
	let name_len = dirent.namelen() as usize;
	let name_bytes = slice::from_raw_parts(name_ptr, name_len);
	let name = crate::NodeName::new_unchecked(name_bytes);
	(dirent, name)
}

#[inline]
pub(crate) unsafe fn write_unchecked<T: Dirent>(
	dirent: T,
	name: &crate::NodeName,
	buf: &mut [u8],
) {
	let buf_ptr = buf.as_mut_ptr();
	let dirent_dst = buf_ptr as *mut T;
	let name_dst = buf_ptr.add(mem::size_of::<T>());

	let name = name.as_bytes();
	let padding_len = (8 - (name.len() % 8)) % 8;

	dirent_dst.write(dirent);
	ptr::copy_nonoverlapping(name.as_ptr(), name_dst, name.len());
	if padding_len > 0 {
		let padding_dst = name_dst.add(name.len());
		ptr::write_bytes(padding_dst, 0, padding_len);
	}
}
