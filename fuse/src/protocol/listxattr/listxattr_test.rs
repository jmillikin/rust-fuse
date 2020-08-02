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

use super::{ListxattrRequest, ListxattrResponse};

#[test]
fn request_sized() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_LISTXATTR)
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 10,
			..Default::default()
		})
		.build_aligned();

	let req: ListxattrRequest = decode_request!(buf);

	assert_eq!(req.size(), Some(10));
}

#[test]
fn request_unsized() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_LISTXATTR)
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 0,
			..Default::default()
		})
		.build_aligned();

	let req: ListxattrRequest = decode_request!(buf);

	assert_eq!(req.size(), None);
}

#[test]
fn response_sized() {
	let mut resp: ListxattrResponse<'static> = todo!();
	/*
	let mut resp = ListxattrRequest {
		header: &HEADER,
		size: 10,
		}.new_response();
		*/
	assert_eq!(resp.request_size, 10);

	// value must fit in kernel buffer
	{
		let cstring = CString::new("12345678901").unwrap();
		let err = resp.push(&cstring).unwrap_err();
		assert_eq!(err.raw_os_error().unwrap(), libc::ERANGE);

		assert!(resp.buf.is_empty());
		assert_eq!(resp.raw.size, 0);
	}

	// pushes append null-terminated xattr names
	{
		let cstring = CString::new("123").unwrap();
		resp.push(&cstring).unwrap();
		assert_eq!(resp.buf, vec![49, 50, 51, 0]);
	}
	{
		let cstring = CString::new("456").unwrap();
		resp.push(&cstring).unwrap();
		assert_eq!(resp.buf, vec![49, 50, 51, 0, 52, 53, 54, 0]);
	}
	assert_eq!(resp.raw.size, 0);

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>() + 8) as u32,
				error: 0,
				unique: 0,
			})
			.push_bytes(&[49, 50, 51, 0, 52, 53, 54, 0])
			.build()
	);
}

#[test]
fn response_unsized() {
	let mut resp: ListxattrResponse<'static> = todo!();
	/*
	let mut resp = ListxattrRequest {
		header: &HEADER,
		size: 0,
		}.new_response();
		*/
	assert_eq!(resp.request_size, 0);

	// set_value() doesn't store value bytes for unsized responses
	{
		let cstring = CString::new("123").unwrap();
		resp.push(&cstring).unwrap();
		assert!(resp.buf.is_empty());
		assert_eq!(resp.raw.size, 4);
	}
	{
		let cstring = CString::new("456").unwrap();
		resp.push(&cstring).unwrap();
		assert!(resp.buf.is_empty());
		assert_eq!(resp.raw.size, 8);
	}

	// size can also be set directly
	resp.set_size(30);
	assert_eq!(resp.raw.size, 30);

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
				size: 30,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_detect_overflow() {
	let mut resp: ListxattrResponse<'static> = todo!();
	/*
		let mut resp = ListxattrRequest {
			header: &HEADER,
			size: 10,
		}.new_response();
	*/
	let big_buf =
		unsafe { slice::from_raw_parts(0 as *const u8, u32::MAX as usize + 1) };
	let big_cstr = unsafe { CStr::from_bytes_with_nul_unchecked(big_buf) };

	let err = resp.push(&big_cstr).unwrap_err();
	assert_eq!(err.raw_os_error().unwrap(), libc::ERANGE);
}
