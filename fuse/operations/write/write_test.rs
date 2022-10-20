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

use fuse::lock;
use fuse::operations::write::{
	WriteRequest,
	WriteRequestFlag,
	WriteResponse,
};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

const DUMMY_WRITE_FLAG: u32 = 0x80000000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&123u64) // fuse_write_in::fh
		.push_sized(&45u64) // fuse_write_in::offset
		.push_sized(&12u32) // fuse_write_in::size
		.push_sized(&0u32) // fuse_write_in::write_flags
		.push_bytes(b"hello.world!")
		.build_aligned();

	let req = decode_request!(WriteRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.lock_owner(), None);
	assert_eq!(req.flags().get(WriteRequestFlag::WRITE_CACHE), false);
	assert_eq!(req.open_flags(), 0);
	assert_eq!(req.value(), b"hello.world!");
}

#[test]
fn request_v7p9() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_write_in {
			fh: 123,
			offset: 45,
			size: 12,
			write_flags: 0,
			lock_owner: 0,
			flags: 67,
			padding: 0,
		})
		.push_bytes(b"hello.world!")
		.build_aligned();

	let req = decode_request!(WriteRequest, buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.lock_owner(), None);
	assert_eq!(req.flags().get(WriteRequestFlag::WRITE_CACHE), false);
	assert_eq!(req.open_flags(), 67);
	assert_eq!(req.value(), b"hello.world!");
}

#[test]
fn request_lock_owner() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_write_in {
			fh: 0,
			offset: 0,
			size: 0,
			write_flags: DUMMY_WRITE_FLAG | fuse_kernel::FUSE_WRITE_LOCKOWNER,
			lock_owner: 123,
			flags: 0,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(WriteRequest, buf);

	assert_eq!(req.lock_owner(), Some(lock::Owner::new(123)));
}

#[test]
fn request_page_cache() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_write_in {
			fh: 0,
			offset: 0,
			size: 0,
			write_flags: DUMMY_WRITE_FLAG | fuse_kernel::FUSE_WRITE_CACHE,
			lock_owner: 0,
			flags: 0,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(WriteRequest, buf);

	assert_eq!(req.flags().get(WriteRequestFlag::WRITE_CACHE), true);
}

#[test]
fn response() {
	let mut resp = WriteResponse::new();
	resp.set_size(123);

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_write_out>()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_write_out {
				size: 123,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = WriteResponse::new();
	response.set_size(123);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"WriteResponse {\n",
			"    size: 123,\n",
			"}",
		),
	);
}
