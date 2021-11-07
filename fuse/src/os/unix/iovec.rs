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

use core::marker::PhantomData;

#[repr(C)]
pub(in crate::os) struct IoVec<'a> {
	iov_base: *const core::ffi::c_void,
	iov_len: usize,
	_phantom: PhantomData<&'a [u8]>,
}

impl IoVec<'static> {
	#[allow(dead_code)]
	pub(in crate::os) fn null() -> Self {
		Self {
			iov_base: core::ptr::null(),
			iov_len: 0,
			_phantom: PhantomData,
		}
	}

	#[allow(dead_code)]
	pub(in crate::os) fn global(buf: &'static [u8]) -> Self {
		IoVec {
			iov_base: buf.as_ptr() as *const core::ffi::c_void,
			iov_len: buf.len(),
			_phantom: PhantomData,
		}
	}
}

impl<'a> IoVec<'a> {
	pub(in crate::os) fn borrow(buf: &'a [u8]) -> Self {
		IoVec {
			iov_base: buf.as_ptr() as *const core::ffi::c_void,
			iov_len: buf.len(),
			_phantom: PhantomData,
		}
	}
}
