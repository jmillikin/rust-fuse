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

use super::{GetxattrRequest, GetxattrResponse};

#[test]
fn request_sized() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_GETXATTR)
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 10,
			..Default::default()
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req: GetxattrRequest = decode_request!(buf);

	let expect = CString::new("hello.world!").unwrap();
	assert_eq!(req.size(), Some(10));
	assert_eq!(req.name(), expect.as_ref());
}

#[test]
fn request_unsized() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_GETXATTR)
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 0,
			..Default::default()
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req: GetxattrRequest = decode_request!(buf);

	let expect = CString::new("hello.world!").unwrap();
	assert_eq!(req.size(), None);
	assert_eq!(req.name(), expect.as_ref());
}

#[test]
fn response_sized() {
	let mut resp: GetxattrResponse<'static> = todo!();
	/*
	let mut resp = GetxattrRequest {
		header: &HEADER,
		raw: &fuse_kernel::fuse_getxattr_in {
			size: 10,
			..Default::default()
		},
		name: CStr::from_bytes_with_nul(b"\x00").unwrap(),
		}.new_response();
		*/
	assert_eq!(resp.request_size, 10);

	// value must fit in kernel buffer
	{
		let err = resp.set_value(&[255; 11]).unwrap_err();
		assert_eq!(err.raw_os_error().unwrap(), libc::ERANGE);

		assert!(resp.buf.is_empty());
		assert_eq!(resp.raw.size, 0);
	}

	resp.set_value(&[255, 0, 255]).unwrap();

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>() + 3) as u32,
				error: 0,
				unique: 0,
			})
			.push_bytes(&[255, 0, 255])
			.build()
	);
}

#[test]
fn response_unsized() {
	let mut resp: GetxattrResponse<'static> = todo!();
	/*
	header: &HEADER,
	raw: &fuse_kernel::fuse_getxattr_in {
		size: 0,
		..Default::default()
	},
	name: CStr::from_bytes_with_nul(b"\x00").unwrap(),
	}.new_response();
	*/
	assert_eq!(resp.request_size, 0);

	// set_value() doesn't store value bytes for unsized responses
	resp.set_value(&[0, 0]).unwrap();
	assert!(resp.buf.is_empty());
	assert_eq!(resp.raw.size, 2);

	resp.set_value(&[0, 0, 0]).unwrap();
	assert!(resp.buf.is_empty());
	assert_eq!(resp.raw.size, 3);

	// size can also be set directly
	resp.set_size(4);
	assert_eq!(resp.raw.size, 4);

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_getxattr_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_getxattr_out {
				size: 4,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_size_clamp() {
	let mut resp: GetxattrResponse<'static> = todo!();
	/*
	let resp = GetxattrRequest {
		header: &HEADER,
		raw: &fuse_kernel::fuse_getxattr_in {
			size: u32::MAX,
			..Default::default()
		},
		name: CStr::from_bytes_with_nul(b"\x00").unwrap(),
		}.new_response();
		*/
	assert_eq!(resp.request_size, 1 << 30);
}

#[test]
fn response_detect_overflow() {
	let mut resp: GetxattrResponse<'static> = todo!();
	/*
	let mut resp = GetxattrRequest {
		header: &HEADER,
		raw: &fuse_kernel::fuse_getxattr_in {
			size: 10,
			..Default::default()
		},
		name: CStr::from_bytes_with_nul(b"\x00").unwrap(),
		}.new_response();
		*/

	let mut buf = vec![0];
	unsafe {
		buf.set_len(u32::MAX as usize + 1)
	};

	let err = resp.set_value(&buf).unwrap_err();
	assert_eq!(err.raw_os_error().unwrap(), libc::ERANGE);
}
