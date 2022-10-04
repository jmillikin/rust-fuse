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

use fuse::operations::getxattr::{GetxattrRequest, GetxattrResponse};
use fuse::xattr;

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request_sized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 10,
			..fuse_kernel::fuse_getxattr_in::zeroed()
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(GetxattrRequest, buf);

	let expect = xattr::Name::new("hello.world!").unwrap();
	assert_eq!(req.size(), Some(num::NonZeroUsize::new(10).unwrap()));
	assert_eq!(req.name(), expect);
}

#[test]
fn request_unsized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 0,
			..fuse_kernel::fuse_getxattr_in::zeroed()
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(GetxattrRequest, buf);

	let expect = xattr::Name::new("hello.world!").unwrap();
	assert_eq!(req.size(), None);
	assert_eq!(req.name(), expect);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, GetxattrRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETXATTR;
			h.nodeid = fuse_kernel::FUSE_ROOT_ID;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 11,
			..fuse_kernel::fuse_getxattr_in::zeroed()
		})
		.push_bytes(b"hello.world!\x00")
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"GetxattrRequest {\n",
			"    node_id: 1,\n",
			"    size: Some(11),\n",
			"    name: \"hello.world!\",\n",
			"}",
		),
	);
}

#[test]
fn response_with_value() {
	let value = xattr::Value::new(&[255, 0, 255]).unwrap();
	let resp = GetxattrResponse::with_value(value);
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
fn response_with_value_size() {
	let resp = GetxattrResponse::with_value_size(4);
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
fn response_with_value_debug() {
	let value = xattr::Value::new(&[1, 2, 3, 4]).unwrap();
	let response = GetxattrResponse::with_value(value);
	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"GetxattrResponse {\n",
			"    value: [\n",
			"        1,\n",
			"        2,\n",
			"        3,\n",
			"        4,\n",
			"    ],\n",
			"}",
		),
	);
}

#[test]
fn response_with_value_size_debug() {
	let response = GetxattrResponse::with_value_size(10);
	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"GetxattrResponse {\n",
			"    size: 10,\n",
			"}",
		),
	);
}
