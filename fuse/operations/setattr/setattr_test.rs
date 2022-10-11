// Copyright 2020 John Millikin and the rust-fuse contributors.
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

use core::mem::size_of;
use core::time;

use fuse::lock;
use fuse::node;
use fuse::operations::setattr::{SetattrRequest, SetattrResponse};

const S_IFREG: u32 = 0o100000;

#[test]
fn request() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetattrRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_SETATTR;
			h.nodeid = 1000;
		})
		.push_sized(&fuse_kernel::fuse_setattr_in {
			valid: 0xFFFF,
			padding: 0,
			fh: 1,
			size: 2,
			lock_owner: 3,
			atime: 4,
			mtime: 5,
			ctime: 6,
			atimensec: 7,
			mtimensec: 8,
			ctimensec: 9,
			mode: S_IFREG | 0o644,
			unused4: 11,
			uid: 12,
			gid: 13,
			unused5: 14,
		})
	});

	assert_eq!(request.node_id(), node::Id::new(1000).unwrap());
	assert_eq!(request.handle(), Some(1));
	assert_eq!(request.size(), Some(2));
	assert_eq!(request.lock_owner(), Some(lock::Owner::new(3)));
	assert_eq!(request.atime(), fuse::UnixTime::new(4, 7));
	assert_eq!(request.atime_now(), true);
	assert_eq!(request.mtime(), fuse::UnixTime::new(5, 8));
	assert_eq!(request.mtime_now(), true);
	assert_eq!(request.ctime(), fuse::UnixTime::new(6, 9));
	assert_eq!(request.user_id(), Some(12));
	assert_eq!(request.group_id(), Some(13));

	let mode = request.mode().unwrap();
	assert_eq!(node::Type::from_mode(mode), Some(node::Type::Regular));
	assert_eq!(mode.permissions(), 0o644);
}

#[test]
fn request_negative_unix_times() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetattrRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_SETATTR;
			h.nodeid = 1000;
		})
		.push_sized(&fuse_kernel::fuse_setattr_in {
			valid: 0xFFFF,
			atime: -4_i64 as u64,
			mtime: -5_i64 as u64,
			ctime: -6_i64 as u64,
			atimensec: 7,
			mtimensec: 8,
			ctimensec: 9,
			..fuse_kernel::fuse_setattr_in::zeroed()
		})
	});

	assert_eq!(request.atime(), fuse::UnixTime::new(-4, 7));
	assert_eq!(request.mtime(), fuse::UnixTime::new(-5, 8));
	assert_eq!(request.ctime(), fuse::UnixTime::new(-6, 9));
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetattrRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_SETATTR;
			h.nodeid = 1000;
		})
		.push_sized(&fuse_kernel::fuse_setattr_in {
			valid: 0xFFFF,
			padding: 0,
			fh: 1,
			size: 2,
			lock_owner: 3,
			atime: 4,
			mtime: 5,
			ctime: 6,
			atimensec: 7,
			mtimensec: 8,
			ctimensec: 9,
			mode: S_IFREG | 0o644,
			unused4: 11,
			uid: 12,
			gid: 13,
			unused5: 14,
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"SetattrRequest {\n",
			"    node_id: 1000,\n",
			"    handle: Some(1),\n",
			"    size: Some(2),\n",
			"    lock_owner: Some(0x0000000000000003),\n",
			"    atime: Some(UnixTime(4.000000007)),\n",
			"    atime_now: true,\n",
			"    mtime: Some(UnixTime(5.000000008)),\n",
			"    mtime_now: true,\n",
			"    ctime: Some(UnixTime(6.000000009)),\n",
			"    mode: Some(0o100644),\n",
			"    user_id: Some(12),\n",
			"    group_id: Some(13),\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let mut response = SetattrResponse::new();
	let attr = response.attr_mut();
	attr.set_node_id(node::Id::new(2).unwrap());
	attr.set_mode(node::Mode::S_IFREG | 0o644);
	attr.set_nlink(1);
	attr.set_size(999);
	response.set_cache_duration(time::Duration::new(123, 456));

	let encoded = fuse_testutil::encode_response!(response, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		fuse_testutil::MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 123,
				attr_valid_nsec: 456,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: 2,
					size: 999,
					mode: S_IFREG | 0o644,
					nlink: 1,
					..fuse_kernel::fuse_attr::zeroed()
				},
			})
			.unpush(
				size_of::<fuse_kernel::fuse_attr_out>()
					- fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE
			)
			.build()
	);
}

#[test]
fn response_v7p9() {
	let mut response = SetattrResponse::new();
	let attr = response.attr_mut();
	attr.set_node_id(node::Id::new(2).unwrap());
	attr.set_mode(node::Mode::S_IFREG | 0o644);
	attr.set_nlink(1);
	attr.set_size(999);
	response.set_cache_duration(time::Duration::new(123, 456));

	let encoded = fuse_testutil::encode_response!(response, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		fuse_testutil::MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_attr_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 123,
				attr_valid_nsec: 456,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: 2,
					size: 999,
					mode: S_IFREG | 0o644,
					nlink: 1,
					..fuse_kernel::fuse_attr::zeroed()
				},
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = SetattrResponse::new();
	let attr = response.attr_mut();
	attr.set_node_id(node::Id::new(2).unwrap());
	attr.set_mode(node::Mode::S_IFREG | 0o644);
	attr.set_nlink(1);
	attr.set_size(999);
	response.set_cache_duration(time::Duration::new(123, 456));

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"SetattrResponse {\n",
			"    attr: NodeAttr {\n",
			"        node_id: Some(2),\n",
			"        size: 999,\n",
			"        blocks: 0,\n",
			"        atime: UnixTime(0.000000000),\n",
			"        mtime: UnixTime(0.000000000),\n",
			"        ctime: UnixTime(0.000000000),\n",
			"        mode: 0o100644,\n",
			"        nlink: 1,\n",
			"        uid: 0,\n",
			"        gid: 0,\n",
			"        rdev: 0,\n",
			"        blksize: 0,\n",
			"    },\n",
			"    cache_duration: 123.000000456s,\n",
			"}",
		),
	);
}
