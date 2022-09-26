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

use fuse::operations::releasedir::{ReleasedirRequest, ReleasedirResponse};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

const DUMMY_RELEASE_FLAG: u32 = 0x80000000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_RELEASEDIR;
			h.nodeid = 123;
		})
		.push_sized(&123u64) // fuse_release_in::fh
		.push_sized(&0xFFu32) // fuse_release_in::flags
		.push_sized(&0u32) // fuse_release_in::padding
		.build_aligned();

	let req = decode_request!(ReleasedirRequest, buf, {
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
			h.opcode = fuse_kernel::FUSE_RELEASEDIR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_release_in {
			fh: 123,
			flags: 0xFF,
			release_flags: 0,
			lock_owner: 0,
		})
		.build_aligned();

	let req = decode_request!(ReleasedirRequest, buf, {
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
			h.opcode = fuse_kernel::FUSE_RELEASEDIR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_release_in {
			fh: 123,
			flags: 0xFF,
			release_flags: DUMMY_RELEASE_FLAG
				| fuse_kernel::FUSE_RELEASE_FLOCK_UNLOCK,
			lock_owner: 123,
		})
		.build_aligned();

	let req = decode_request!(ReleasedirRequest, buf, {
		protocol_version: (7, 8),
	});

	assert_eq!(req.lock_owner(), Some(123));
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, ReleasedirRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_RELEASEDIR;
			h.nodeid = fuse_kernel::FUSE_ROOT_ID;
		})
		.push_sized(&fuse_kernel::fuse_release_in {
			fh: 3,
			flags: 0x4,
			release_flags: 0,
			lock_owner: 0,
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ReleasedirRequest {\n",
			"    node_id: 1,\n",
			"    handle: 3,\n",
			"    lock_owner: None,\n",
			"    open_flags: 0x00000004,\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let resp = ReleasedirResponse::new();
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: size_of::<fuse_kernel::fuse_out_header>() as u32,
				error: 0,
				unique: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let response = ReleasedirResponse::new();
	assert_eq!(format!("{:#?}", response), "ReleasedirResponse");
}
