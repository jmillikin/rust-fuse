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

use core::marker::PhantomData;
use core::mem::size_of;

use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{ReleaseRequest, ReleaseResponse};

const DUMMY_RELEASE_FLAG: u32 = 0x80000000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_RELEASE;
			h.nodeid = 123;
		})
		.push_sized(&super::fuse_release_in_v7p1 {
			fh: 123,
			flags: 0xFF,
			padding: 0,
		})
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
			h.opcode = fuse_kernel::FUSE_RELEASE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_release_in {
			fh: 123,
			flags: 0xFF,
			release_flags: 0,
			lock_owner: 0,
		})
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
			h.opcode = fuse_kernel::FUSE_RELEASE;
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

	let req = decode_request!(ReleaseRequest, buf, {
		protocol_version: (7, 8),
	});

	assert_eq!(req.lock_owner(), Some(123));
}

#[test]
fn request_impl_debug() {
	let request = &ReleaseRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
		handle: 3,
		lock_owner: None,
		open_flags: 0x4,
	};

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

#[test]
fn response() {
	let resp = ReleaseResponse::new();
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
	let response = ReleaseResponse::new();
	assert_eq!(format!("{:#?}", response), "ReleaseResponse");
}
