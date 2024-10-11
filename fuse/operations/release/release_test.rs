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
use fuse::server::ReleaseRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

const DUMMY_RELEASE_FLAG: u32 = 0x80000000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RELEASE;
			h.nodeid = 123;
		})
		.push_sized(&123u64) // fuse_release_in::fh
		.push_sized(&0xFFu32) // fuse_release_in::flags
		.push_sized(&0u32) // fuse_release_in::padding
		.build_aligned();

	let req = decode_request!(ReleaseRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.open_flags(), 0xFF);
	assert_eq!(req.lock_owner(), None);
}

#[test]
fn request_v7p8() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RELEASE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_release_in {
			fh: 123,
			flags: 0xFF,
		}))
		.build_aligned();

	let req = decode_request!(ReleaseRequest, buf, {
		protocol_version: (7, 8),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.open_flags(), 0xFF);
	assert_eq!(req.lock_owner(), None);
}

#[test]
fn request_lock_owner() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RELEASE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_release_in {
			fh: 123,
			flags: 0xFF,
			release_flags: DUMMY_RELEASE_FLAG
				| kernel::FUSE_RELEASE_FLOCK_UNLOCK,
			lock_owner: 123,
		}))
		.build_aligned();

	let req = decode_request!(ReleaseRequest, buf, {
		protocol_version: (7, 8),
	});

	assert_eq!(req.lock_owner(), Some(fuse::LockOwner(123)));
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, ReleaseRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RELEASE;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_release_in {
			fh: 3,
			flags: 0x4,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ReleaseRequest {\n",
			"    node_id: 1,\n",
			"    handle: 3,\n",
			"    lock_owner: None,\n",
			"    open_flags: 0x00000004,\n",
			"}",
		),
	);
}
