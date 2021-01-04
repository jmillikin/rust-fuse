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

use std::ffi::{CStr, CString};

use test_client_base::{dirent_type_name, errno_name};

fn main() {
	println!("START {}", std::env::args().next().unwrap());

	let path = CString::new("/rust-fuse/testfs/readdir.d").unwrap();
	let dir_p = unsafe { libc::opendir(path.as_ptr()) };
	println!(
		"\nopendir({:?}) -> {}",
		path,
		if dir_p.is_null() { "NULL" } else { "*DIR" }
	);
	println!("  errno: {}", errno_name());

	if dir_p.is_null() {
		std::process::exit(1);
	}

	let print_next_dirent = || {
		unsafe {
			let dirent_p = libc::readdir(dir_p);
			if dirent_p.is_null() {
				println!("\nreaddir() -> NULL");
				println!("  errno: {}", errno_name());
			} else {
				println!("\nreaddir() -> *dirent");
				println!("  errno: {}", errno_name());

				let dirent: &libc::dirent = &(*dirent_p);
				let name = CStr::from_ptr(dirent.d_name.as_ptr());
				println!("");
				println!("  dirent_p.d_ino: {}", dirent.d_ino);
				println!("  dirent_p.d_off: {}", dirent.d_off);
				println!("  dirent_p.d_reclen: {}", dirent.d_reclen);
				println!("  dirent_p.d_type: {}", dirent_type_name(dirent));
				println!("  dirent_p.d_name: {:?}", name);
			}
		};
	};

	print_next_dirent();

	let pos = unsafe { libc::telldir(dir_p) };
	println!("\ntelldir() -> {}", pos);
	println!("  errno: {}", errno_name());

	print_next_dirent();

	unsafe {
		libc::seekdir(dir_p, pos)
	};
	println!("\nseekdir()");
	println!("  errno: {}", errno_name());

	print_next_dirent();
	print_next_dirent();

	let rc = unsafe { libc::closedir(dir_p) };
	println!("\nclosedir() -> {}", rc);
	println!("  errno: {}", errno_name());
}
