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
		.set_opcode(fuse_kernel::FUSE_READ)
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
	assert_eq!(req.flags(), 0);
}

#[test]
fn request_v7p9() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_READ)
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
	assert_eq!(req.flags(), 67);
}

#[test]
fn request_lock_owner() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_READ)
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
fn response() {
	let mut resp: ReadResponse = todo!();
	/*
	let mut resp = ReadRequest {
		header: &HEADER,
		handle: 0,
		offset: 0,
		size: 10,
		read_flags: 0,
		lock_owner: 0,
		flags: 0,
		}.new_response();
		*/
	assert_eq!(resp.request_size, 10);

	// value must fit in kernel buffer
	{
		let err = resp.set_value(&[255; 11]).unwrap_err();
		assert_eq!(err.raw_os_error().unwrap(), errors::ERANGE.get() as i32);

		assert!(resp.buf.is_empty());
	}

	resp.set_value(&[255, 0, 255]).unwrap();
	assert_eq!(resp.buf, &[255, 0, 255]);

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ resp.buf.len()) as u32,
				error: 0,
				unique: 0,
			})
			.push_bytes(&[255, 0, 255])
			.build()
	);
}

#[test]
fn response_detect_overflow() {
	let mut resp: ReadResponse = todo!();
	/*
		let mut resp = ReadRequest {
			header: &HEADER,
			handle: 0,
			offset: 0,
			size: 10,
			read_flags: 0,
			lock_owner: 0,
			flags: 0,
		}.new_response();
	*/
	// response must be small enough to size with a u32
	let mut buf = vec![0];
	unsafe {
		buf.set_len(u32::MAX as usize + 1)
	};

	let err = resp.set_value(&buf).unwrap_err();
	assert_eq!(err.raw_os_error().unwrap(), errors::ERANGE.get() as i32);

	assert!(resp.buf.is_empty());
}
