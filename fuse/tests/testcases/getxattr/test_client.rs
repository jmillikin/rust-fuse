// Copyright 2021 John Millikin and the rust-fuse contributors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//		 http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::ffi::CString;

use test_client_base::errno_name;

fn main() {
	println!("START {}", std::env::args().next().unwrap());

	let path = CString::new("/rust-fuse/testfs/xattrs.txt").unwrap();
	let xattr_small = CString::new("xattr_small").unwrap();
	let xattr_noexist = CString::new("xattr_noexist").unwrap();
	let xattr_toobig = CString::new("xattr_toobig").unwrap();

	{
		let mut value = [0u8; 32];
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				xattr_small.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				0,
			)
		};
		println!("\ngetxattr({:?}, {:?}, _, 0) -> {}", path, xattr_small, rc);
		println!("  errno: {}", errno_name());
		println!("  value: {:?}", value);
	}

	{
		let mut value = [0u8; 32];
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				xattr_small.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				32,
			)
		};
		println!("\ngetxattr({:?}, {:?}, _, 32) -> {}", path, xattr_small, rc);
		println!("  errno: {}", errno_name());
		println!("  value: {:?}", value);
	}

	{
		let mut value = [0u8; 32];
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				xattr_noexist.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				32,
			)
		};
		println!(
			"\ngetxattr({:?}, {:?}, _, 32) -> {}",
			path, xattr_noexist, rc
		);
		println!("  errno: {}", errno_name());
	}

	{
		let mut value = [0u8; 32];
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				xattr_small.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				1,
			)
		};
		println!("\ngetxattr({:?}, {:?}, _, 1) -> {}", path, xattr_small, rc);
		println!("  errno: {}", errno_name());
	}

	{
		let mut value = [0u8; 32];
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				xattr_toobig.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				32,
			)
		};
		println!(
			"\ngetxattr({:?}, {:?}, _, 32) -> {}",
			path, xattr_toobig, rc
		);
		println!("  errno: {}", errno_name());
	}
}
