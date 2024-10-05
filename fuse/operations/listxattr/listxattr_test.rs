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
use core::num;

use fuse::operations::listxattr::{
	ListxattrNamesWriter,
	ListxattrRequest,
	ListxattrResponse,
};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request_sized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LISTXATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 10,
			..fuse_kernel::fuse_getxattr_in::zeroed()
		})
		.build_aligned();

	let req = decode_request!(ListxattrRequest, buf);

	assert_eq!(req.size(), Some(num::NonZeroUsize::new(10).unwrap()));
}

#[test]
fn request_unsized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LISTXATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 0,
			..fuse_kernel::fuse_getxattr_in::zeroed()
		})
		.build_aligned();

	let req = decode_request!(ListxattrRequest, buf);

	assert_eq!(req.size(), None);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, ListxattrRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LISTXATTR;
			h.nodeid = fuse_kernel::FUSE_ROOT_ID;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 11,
			..fuse_kernel::fuse_getxattr_in::zeroed()
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ListxattrRequest {\n",
			"    node_id: 1,\n",
			"    size: Some(11),\n",
			"}",
		),
	);
}

#[test]
fn response_with_names() {
	let mut buf = [0u8; 10];
	let mut names = ListxattrNamesWriter::new(&mut buf);
	assert_eq!(names.capacity(), 10);

	// response must fit in provided buffer
	{
		let name = fuse::XattrName::new("12345678901").unwrap();
		assert!(names.try_push(name).is_err());
	}

	// xattr names are NUL-terminated, so two 3-byte names requires 8 bytes
	// of buffer space.
	names.try_push(fuse::XattrName::new("123").unwrap()).unwrap();
	names.try_push(fuse::XattrName::new("456").unwrap()).unwrap();
	assert_eq!(names.position(), 8);

	let resp = ListxattrResponse::with_names(names);
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>() + 8) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_bytes(&[49, 50, 51, 0, 52, 53, 54, 0])
			.build()
	);
}

#[test]
fn response_with_names_size() {
	let resp = ListxattrResponse::with_names_size(8);
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_getxattr_out>()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_getxattr_out {
				size: 8,
				padding: 0,
			})
			.build()
	);
}

#[cfg(target_os = "linux")]
#[test]
fn response_size_limit() {
	// listxattr response size can't exceed XATTR_LIST_MAX
	let mut buf = [0u8; 65536 + 1];
	let names = ListxattrNamesWriter::new(&mut buf);
	assert_eq!(names.capacity(), 65536);
}

#[test]
fn response_with_names_debug() {
	let mut buf = [0u8; 10];
	let mut names = ListxattrNamesWriter::new(&mut buf);

	names.try_push(fuse::XattrName::new("123").unwrap()).unwrap();
	names.try_push(fuse::XattrName::new("456").unwrap()).unwrap();

	let response = ListxattrResponse::with_names(names);
	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ListxattrResponse {\n",
			"    names: [\n",
			"        \"123\",\n",
			"        \"456\",\n",
			"    ],\n",
			"}",
		),
	);
}

#[test]
fn response_with_names_size_debug() {
	let response = ListxattrResponse::with_names_size(8);
	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ListxattrResponse {\n",
			"    size: 8,\n",
			"}",
		),
	);
}
