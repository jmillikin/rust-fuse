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
	assert_eq!(req.size(), Some(num::NonZeroU32::new(10).unwrap()));
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
fn response_sized() {
	let request_size = num::NonZeroU32::new(10);
	let mut resp = GetxattrResponse::new(request_size);
	assert_eq!(resp.request_size(), request_size);

	// value must fit in kernel buffer
	assert!(resp.try_set_value(&[255; 11]).is_err());

	resp.set_value(&[255, 0, 255]);

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
	let mut resp = GetxattrResponse::new(None);
	assert_eq!(resp.request_size(), None);

	// set_value() doesn't allow value sizes larger than XATTR_SIZE_MAX
	let mut val_toobig = Vec::new();
	if let Some(max_len) = xattr::Value::MAX_LEN {
		val_toobig.resize(max_len + 1, 0u8);
		assert!(resp.try_set_value(&val_toobig).is_err());
		assert!(resp.value().is_empty());
		//assert_eq!(resp.raw.size, 0);
	}

	// set_value() doesn't store value bytes for unsized responses
	resp.set_value(&[1, 2, 3, 4]);
	assert!(resp.value().is_empty());
	//assert_eq!(resp.raw.size, 4);

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
fn response_impl_debug() {
	let request_size = num::NonZeroU32::new(10);
	let mut response = GetxattrResponse::new(request_size);
	response.set_value(b"some\x00bytes");

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"GetxattrResponse {\n",
			"    request_size: Some(10),\n",
			"    value: \"some\\x00bytes\",\n",
			"}",
		),
	);
}
