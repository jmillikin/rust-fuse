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

	let path_old = CString::new("/rust-fuse/testfs/rename_old.txt").unwrap();
	let path_new = CString::new("/rust-fuse/testfs/rename_new.txt").unwrap();
	let path_noexist =
		CString::new("/rust-fuse/testfs/rename_noexist.txt").unwrap();
	let path_dir = CString::new("/rust-fuse/testfs/rename_dir.d").unwrap();

	{
		let rc = unsafe { libc::rename(path_old.as_ptr(), path_new.as_ptr()) };
		println!("\nrename({:?}, {:?}) -> {}", path_old, path_new, rc);
		println!("  errno: {}", errno_name());
	}

	{
		let rc =
			unsafe { libc::rename(path_noexist.as_ptr(), path_new.as_ptr()) };
		println!("\nrename({:?}, {:?}) -> {}", path_noexist, path_new, rc);
		println!("  errno: {}", errno_name());
	}

	{
		let rc = unsafe { libc::rename(path_old.as_ptr(), path_dir.as_ptr()) };
		println!("\nrename({:?}, {:?}) -> {}", path_old, path_dir, rc);
		println!("  errno: {}", errno_name());
	}
}
