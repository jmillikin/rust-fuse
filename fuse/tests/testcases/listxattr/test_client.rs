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
	let path_toobig =
		CString::new("/rust-fuse/testfs/xattrs_toobig.txt").unwrap();

	{
		let mut name_list = [0i8; 32];
		let rc = unsafe {
			libc::listxattr(path.as_ptr(), name_list.as_mut_ptr(), 0)
		};
		println!("\nlistxattr({:?}, _, 0) -> {}", path, rc);
		println!("  errno: {}", errno_name());
		println!("  value: {:?}", name_list);
	}

	{
		let mut name_list = [0i8; 32];
		let rc = unsafe {
			libc::listxattr(path.as_ptr(), name_list.as_mut_ptr(), 32)
		};
		println!("\nlistxattr({:?}, _, 32) -> {}", path, rc);
		println!("  errno: {}", errno_name());
		println!("  value: {:?}", name_list);
	}

	{
		let mut name_list = [0i8; 32];
		let rc = unsafe {
			libc::listxattr(path.as_ptr(), name_list.as_mut_ptr(), 1)
		};
		println!("\nlistxattr({:?}, _, 1) -> {}", path, rc);
		println!("  errno: {}", errno_name());
	}

	{
		let mut name_list = [0i8; 32];
		let rc = unsafe {
			libc::listxattr(path_toobig.as_ptr(), name_list.as_mut_ptr(), 32)
		};
		println!("\nlistxattr({:?}, _, 1) -> {}", path_toobig, rc);
		println!("  errno: {}", errno_name());
	}
}
