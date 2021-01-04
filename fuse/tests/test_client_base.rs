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

pub fn errno() -> libc::c_int {
	unsafe { *libc::__errno_location() }
}

pub fn errno_name() -> String {
	match errno() {
		libc::E2BIG => "E2BIG".to_string(),
		libc::EEXIST => "EEXIST".to_string(),
		libc::EIO => "EIO".to_string(),
		libc::EISDIR => "EISDIR".to_string(),
		#[allow(deprecated)]
		libc::ENOATTR => "ENOATTR".to_string(),
		libc::ENOENT => "ENOENT".to_string(),
		libc::ENOSYS => "ENOSYS".to_string(),
		libc::ERANGE => "ERANGE".to_string(),
		x => format!("{}", x),
	}
}

pub fn dirent_type_name(dirent: &libc::dirent) -> String {
	match dirent.d_type {
		libc::DT_BLK => "DT_BLK".to_string(),
		libc::DT_CHR => "DT_CHR".to_string(),
		libc::DT_DIR => "DT_DIR".to_string(),
		libc::DT_FIFO => "DT_FIFO".to_string(),
		libc::DT_LNK => "DT_LNK".to_string(),
		libc::DT_REG => "DT_REG".to_string(),
		libc::DT_SOCK => "DT_SOCK".to_string(),
		libc::DT_UNKNOWN => "DT_UNKNOWN".to_string(),
		x => format!("{}", x),
	}
}
