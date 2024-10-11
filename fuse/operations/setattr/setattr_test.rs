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

use fuse::kernel;
use fuse::server::SetattrRequest;

use fuse_testutil as testutil;

const S_IFREG: u32 = 0o100000;

#[test]
fn request() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetattrRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SETATTR;
			h.nodeid = 1000;
		})
		.push_sized(&testutil::new!(kernel::fuse_setattr_in {
			valid: 0xFFFF,
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
			uid: 12,
			gid: 13,
		}))
	});

	assert_eq!(request.node_id(), fuse::NodeId::new(1000).unwrap());
	assert_eq!(request.handle(), Some(1));
	assert_eq!(request.size(), Some(2));
	assert_eq!(request.lock_owner(), Some(fuse::LockOwner(3)));
	assert_eq!(request.atime(), fuse::UnixTime::new(4, 7));
	assert_eq!(request.atime_now(), true);
	assert_eq!(request.mtime(), fuse::UnixTime::new(5, 8));
	assert_eq!(request.mtime_now(), true);
	assert_eq!(request.ctime(), fuse::UnixTime::new(6, 9));
	assert_eq!(request.user_id(), Some(12));
	assert_eq!(request.group_id(), Some(13));

	let mode = request.mode().unwrap();
	assert_eq!(fuse::FileType::from_mode(mode), Some(fuse::FileType::Regular));
	assert_eq!(mode.permissions(), 0o644);
}

#[test]
fn request_negative_unix_times() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetattrRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SETATTR;
			h.nodeid = 1000;
		})
		.push_sized(&testutil::new!(kernel::fuse_setattr_in {
			valid: 0xFFFF,
			atime: -4_i64 as u64,
			mtime: -5_i64 as u64,
			ctime: -6_i64 as u64,
			atimensec: 7,
			mtimensec: 8,
			ctimensec: 9,
		}))
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
			h.opcode = kernel::fuse_opcode::FUSE_SETATTR;
			h.nodeid = 1000;
		})
		.push_sized(&testutil::new!(kernel::fuse_setattr_in {
			valid: 0xFFFF,
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
			uid: 12,
			gid: 13,
		}))
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
