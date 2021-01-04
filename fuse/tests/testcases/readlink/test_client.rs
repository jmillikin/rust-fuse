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

	{
		let path = CString::new("/rust-fuse/testfs/symlink.txt").unwrap();

		let mut value = [0i8; 32];
		let rc =
			unsafe { libc::readlink(path.as_ptr(), value.as_mut_ptr(), 32) };
		println!("\nreadlink({:?}, _, 32) -> {}", path, rc);
		println!("  errno: {}", errno_name());
		println!("  value: {:?}", value);
	}

	{
		let path = CString::new("/rust-fuse/testfs/noexist.txt").unwrap();

		let mut value = [0i8; 32];
		let rc =
			unsafe { libc::readlink(path.as_ptr(), value.as_mut_ptr(), 32) };
		println!("\nreadlink({:?}, _, 32) -> {}", path, rc);
		println!("  errno: {}", errno_name());
	}
}
