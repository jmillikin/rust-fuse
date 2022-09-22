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

#[cfg(feature = "std")]
use std::ffi::{CStr, CString};

mod linux;

use crate::linux::{MountData, MountOptions};

#[test]
fn opt_allow_other() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.allow_other(), false);
	opts.set_allow_other(true);
	assert_eq!(opts.allow_other(), true);
}

#[test]
fn opt_block_size() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.block_size(), None);
	opts.set_block_size(Some(123u32));
	assert_eq!(opts.block_size(), Some(123u32));
}

#[test]
fn opt_default_permissions() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.default_permissions(), false);
	opts.set_default_permissions(true);
	assert_eq!(opts.default_permissions(), true);
}

#[cfg(feature = "std")]
#[test]
fn opt_fs_subtype() {
	let mut opts = MountOptions::new();
	let fs_subtype = CString::new("rust_fuse_subtype").unwrap();

	assert_eq!(opts.fs_subtype(), None);
	opts.set_fs_subtype(Some(&fs_subtype));
	assert_eq!(opts.fs_subtype(), Some(fs_subtype.as_ref()));
}

#[cfg(feature = "std")]
#[test]
fn opt_fs_type() {
	let mut opts = MountOptions::new();
	let fuse = CString::new("fuse").unwrap();
	let fs_type = CString::new("rust_fuse_type").unwrap();

	assert_eq!(opts.fs_type(), fuse.as_ref());
	opts.set_fs_type(Some(&fs_type));
	assert_eq!(opts.fs_type(), fs_type.as_ref());
}

#[test]
fn opt_fuse_device_fd() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.fuse_device_fd(), None);
	opts.set_fuse_device_fd(Some(123u32));
	assert_eq!(opts.fuse_device_fd(), Some(123u32));
}

#[test]
fn opt_group_id() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.group_id(), None);
	opts.set_group_id(Some(123u32));
	assert_eq!(opts.group_id(), Some(123u32));
}

#[test]
fn opt_max_read() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.max_read(), None);
	opts.set_max_read(Some(123u32));
	assert_eq!(opts.max_read(), Some(123u32));
}

#[test]
fn opt_root_mode() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.root_mode(), None);
	opts.set_root_mode(Some(123u32));
	assert_eq!(opts.root_mode(), Some(123u32));
}

#[cfg(feature = "std")]
#[test]
fn opt_source() {
	let mut opts = MountOptions::new();
	let fuse = CString::new("fuse").unwrap();
	let source = CString::new("rust_fuse_source").unwrap();

	assert_eq!(opts.source(), fuse.as_ref());
	opts.set_source(Some(&source));
	assert_eq!(opts.source(), source.as_ref());
}

#[test]
fn opt_user_id() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.user_id(), None);
	opts.set_user_id(Some(123u32));
	assert_eq!(opts.user_id(), Some(123u32));
}

#[test]
fn mount_data() {
	#[cfg(feature = "std")]
	let fs_subtype = CString::new("rust_fuse_subtype").unwrap();

	let mut opts = MountOptions::new();

	opts.set_allow_other(true);
	opts.set_block_size(Some(10));
	opts.set_default_permissions(true);
	#[cfg(feature = "std")]
	opts.set_fs_subtype(Some(&fs_subtype));
	opts.set_fuse_device_fd(Some(20));
	opts.set_group_id(Some(30));
	opts.set_max_read(Some(40));
	opts.set_root_mode(Some(50));
	opts.set_user_id(Some(60));

	let mut buf = [0u8; 512];
	let mount_data = MountData::new(&mut buf, &opts).unwrap();

	#[cfg(feature = "std")]
	let expect = concat!(
		"fd=20,",
		"allow_other,",
		"blksize=10,",
		"default_permissions,",
		"group_id=30,",
		"max_read=40,",
		"rootmode=62,",
		"subtype=rust_fuse_subtype,",
		"user_id=60\0",
	).as_bytes();

	#[cfg(not(feature = "std"))]
	let expect = concat!(
		"fd=20,",
		"allow_other,",
		"blksize=10,",
		"default_permissions,",
		"group_id=30,",
		"max_read=40,",
		"rootmode=62,",
		"user_id=60\0",
	).as_bytes();

	#[cfg(feature = "std")]
	let expect_cstr = CStr::from_bytes_with_nul(expect).unwrap();
	#[cfg(feature = "std")]
	assert_eq!(mount_data.as_cstr(), expect_cstr);
	assert_eq!(mount_data.as_bytes_with_nul(), expect);
}

#[test]
fn mount_data_empty() {
	let opts = MountOptions::new();
	let mut buf = [0u8; 512];
	let mount_data = MountData::new(&mut buf, &opts).unwrap();

	assert_eq!(mount_data.as_bytes_with_nul(), b"\0");
}

#[test]
fn mount_data_small_buf() {
	let mut opts = MountOptions::new();
	opts.set_fuse_device_fd(Some(20));

	{
		let mut buf = [0u8; 0];
		let mount_data = MountData::new(&mut buf, &opts);
		assert!(mount_data.is_none());
	}
	{
		let mut buf = [0u8; 5];
		let mount_data = MountData::new(&mut buf, &opts);
		assert!(mount_data.is_none());
	}
	{
		let mut buf = [0u8; 6];
		let mount_data = MountData::new(&mut buf, &opts);
		assert!(mount_data.is_some());
		assert_eq!(mount_data.unwrap().as_bytes_with_nul(), b"fd=20\0");
	}
}

#[cfg(feature = "std")]
#[test]
fn mount_data_ignore_empty_subtype() {
	let empty_subtype = CString::new("").unwrap();

	let mut opts = MountOptions::new();
	opts.set_fs_subtype(Some(&empty_subtype));

	let mut buf = [0u8; 512];
	let mount_data = MountData::new(&mut buf, &opts).unwrap();
	assert_eq!(mount_data.as_bytes_with_nul(), b"\0");
}

#[cfg(feature = "std")]
#[test]
fn mount_data_reject_subtype_with_comma() {
	let fs_subtype = CString::new("bad,subtype").unwrap();

	let mut opts = MountOptions::new();
	opts.set_fs_subtype(Some(&fs_subtype));

	let mut buf = [0u8; 512];
	let mount_data = MountData::new(&mut buf, &opts);
	assert!(mount_data.is_none());
}

#[cfg(feature = "std")]
#[test]
fn mount_data_no_source_or_fs_type() {
	let fs_type = CString::new("rust_fuse_type").unwrap();
	let source = CString::new("rust_fuse_source").unwrap();

	let mut opts = MountOptions::new();
	opts.set_fs_type(Some(&fs_type));
	opts.set_source(Some(&source));

	let mut buf = [0u8; 512];
	let mount_data = MountData::new(&mut buf, &opts).unwrap();
	assert_eq!(mount_data.as_bytes_with_nul(), b"\0");
}
