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

use crate::internal::testutil::MessageBuilder;
use crate::protocol::prelude::*;

use super::{ReadRequest, ReadResponse};

const DUMMY_READ_FLAG: u32 = 0x80000000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READ;
			h.nodeid = 123;
		})
		.push_sized(&super::fuse_read_in_v7p1 {
			fh: 123,
			offset: 45,
			size: 12,
			padding: 0,
		})
		.build_aligned();

	let req: ReadRequest = decode_request!(buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.size(), 12);
	assert_eq!(req.lock_owner(), None);
	assert_eq!(req.open_flags(), 0);
}

#[test]
fn request_v7p9() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READ;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 12,
			read_flags: 0,
			lock_owner: 0,
			flags: 67,
			padding: 0,
		})
		.build_aligned();

	let req: ReadRequest = decode_request!(buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.size(), 12);
	assert_eq!(req.lock_owner(), None);
	assert_eq!(req.open_flags(), 67);
}

#[test]
fn request_lock_owner() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READ;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 12,
			read_flags: DUMMY_READ_FLAG | fuse_kernel::FUSE_READ_LOCKOWNER,
			lock_owner: 123,
			flags: 67,
			padding: 0,
		})
		.build_aligned();

	let req: ReadRequest = decode_request!(buf);

	assert_eq!(req.lock_owner(), Some(123));
}

#[test]
fn request_impl_debug() {
	let request = &ReadRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
		size: 1,
		offset: 2,
		handle: 3,
		lock_owner: None,
		open_flags: 0x4,
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ReadRequest {\n",
			"    node_id: 1,\n",
			"    size: 1,\n",
			"    offset: 2,\n",
			"    handle: 3,\n",
			"    lock_owner: None,\n",
			"    open_flags: 0x00000004,\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let resp = ReadResponse::from_bytes(&[255, 0, 255]);
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ resp.bytes.len()) as u32,
				error: 0,
				unique: 0,
			})
			.push_bytes(&[255, 0, 255])
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let response = ReadResponse::from_bytes(&[255, 0, 255]);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ReadResponse {\n",
			r#"    bytes: "\xff\x00\xff","#, "\n",
			"}",
		),
	);
}
