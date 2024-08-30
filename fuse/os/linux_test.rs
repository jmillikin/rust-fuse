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

use std::ffi::CString;

mod linux;

use crate::linux::{
	FuseSubtype,
	MountOptions,
	MountSource,
	MountType,
	mount_data,
};

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

#[test]
fn opt_subtype() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.subtype(), None);
	opts.set_subtype(Some(FuseSubtype::new(c"rust_subtype").unwrap()));
	assert_eq!(opts.subtype().unwrap().as_cstr(), subtype.as_ref());
}

#[test]
fn opt_mount_source() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.mount_source(), MountSource::FUSE);
	opts.set_mount_source(MountSource::new(c"rust_fuse_source").unwrap());
	assert_eq!(opts.mount_source().as_cstr(), source.as_ref());
}

#[test]
fn opt_mount_type() {
	let mut opts = MountOptions::new();

	assert_eq!(MountType::FUSE.as_cstr().to_bytes(), b"fuse");
	assert_eq!(MountType::FUSEBLK.as_cstr().to_bytes(), b"fuseblk");

	assert_eq!(opts.mount_type(), MountType::FUSE);
	opts.set_mount_type(MountType::FUSEBLK);
	assert_eq!(opts.mount_type(), MountType::FUSEBLK);
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

#[test]
fn opt_user_id() {
	let mut opts = MountOptions::new();

	assert_eq!(opts.user_id(), None);
	opts.set_user_id(Some(123u32));
	assert_eq!(opts.user_id(), Some(123u32));
}

#[test]
fn mount_data_full() {
	let mut opts = MountOptions::new();

	opts.set_allow_other(true);
	opts.set_block_size(Some(10));
	opts.set_default_permissions(true);
	opts.set_subtype(FuseSubtype::new(c"rust_subtype"));
	opts.set_fuse_device_fd(Some(20));
	opts.set_group_id(Some(30));
	opts.set_max_read(Some(40));
	opts.set_root_mode(Some(50));
	opts.set_user_id(Some(60));

	let expect = concat!(
		"fd=20,",
		"allow_other,",
		"blksize=10,",
		"default_permissions,",
		"group_id=30,",
		"max_read=40,",
		"rootmode=62,",
		"subtype=rust_subtype,",
		"user_id=60\0",
	).as_bytes();

	let mut buf = [0u8; 512];
	assert_eq!(mount_data(&opts, &mut buf), Some(expect));
}

#[test]
fn mount_data_empty() {
	let opts = MountOptions::new();

	let mut buf = [0u8; 512];
	assert_eq!(mount_data(&opts, &mut buf), Some(&b"\0"[..]));
}

#[test]
fn mount_data_small_buf() {
	let mut opts = MountOptions::new();
	opts.set_fuse_device_fd(Some(20));

	{
		let mut buf = [0u8; 0];
		assert!(mount_data(&opts, &mut buf).is_none());
	}
	{
		let mut buf = [0u8; 5];
		assert!(mount_data(&opts, &mut buf).is_none());
	}
	{
		let mut buf = [0u8; 6];
		assert_eq!(mount_data(&opts, &mut buf), Some(&b"fd=20\0"[..]));
	}
}

#[test]
fn subtype_new() {
	assert!(FuseSubtype::new(c"").is_none());
	assert!(FuseSubtype::new(c"bad,subtype").is_none());
}
