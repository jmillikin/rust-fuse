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
use fuse::server::WriteRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

const DUMMY_WRITE_FLAG: u32 = 0x80000000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_WRITE;
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
	assert_eq!(req.flags().get(fuse::WriteRequestFlag::WRITE_CACHE), false);
	assert_eq!(req.open_flags(), 0);
	assert_eq!(req.value(), b"hello.world!");
}

#[test]
fn request_v7p9() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_write_in {
			fh: 123,
			offset: 45,
			size: 12,
			flags: 67,
		}))
		.push_bytes(b"hello.world!")
		.build_aligned();

	let req = decode_request!(WriteRequest, buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.lock_owner(), None);
	assert_eq!(req.flags().get(fuse::WriteRequestFlag::WRITE_CACHE), false);
	assert_eq!(req.open_flags(), 67);
	assert_eq!(req.value(), b"hello.world!");
}

#[test]
fn request_lock_owner() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_write_in {
			write_flags: DUMMY_WRITE_FLAG | kernel::FUSE_WRITE_LOCKOWNER,
			lock_owner: 123,
		}))
		.build_aligned();

	let req = decode_request!(WriteRequest, buf);

	assert_eq!(req.lock_owner(), Some(fuse::LockOwner(123)));
}

#[test]
fn request_page_cache() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_write_in {
			write_flags: DUMMY_WRITE_FLAG | kernel::FUSE_WRITE_CACHE,
		}))
		.build_aligned();

	let req = decode_request!(WriteRequest, buf);

	assert_eq!(req.flags().get(fuse::WriteRequestFlag::WRITE_CACHE), true);
}
