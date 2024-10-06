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

use fuse::kernel;
use fuse::operations::setxattr::{SetxattrRequest, SetxattrResponse};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&10u32) // fuse_setxattr_in::size
		.push_sized(&0b11u32) // fuse_setxattr_in::flags
		.push_bytes(b"hello.world!\x00")
		.push_bytes(b"some\x00value")
		.build_aligned();

	let req = decode_request!(SetxattrRequest, buf);

	let expect_name = fuse::XattrName::new("hello.world!").unwrap();
	assert_eq!(req.name(), expect_name);
	assert_eq!(req.value().as_bytes(), b"some\x00value");
	assert_eq!(req.setxattr_flags(), 0b11);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetxattrRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SETXATTR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&10u32) // fuse_setxattr_in::size
		.push_sized(&0u32) // fuse_setxattr_in::flags
		.push_bytes(b"hello.world!\x00")
		.push_bytes(b"some\x00value")
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"SetxattrRequest {\n",
			"    node_id: 1,\n",
			"    name: \"hello.world!\",\n",
			"    flags: SetxattrRequestFlags {},\n",
			"    setxattr_flags: 0x00000000,\n",
			"    value: [\n",
			"        115,\n",
			"        111,\n",
			"        109,\n",
			"        101,\n",
			"        0,\n",
			"        118,\n",
			"        97,\n",
			"        108,\n",
			"        117,\n",
			"        101,\n",
			"    ],\n",
			"}",
		),
	);
}

#[test]
fn response_empty() {
	let resp = SetxattrResponse::new();
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_out_header {
				len: size_of::<kernel::fuse_out_header>() as u32,
				unique: 0xAABBCCDD,
			}))
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let response = SetxattrResponse::new();

	assert_eq!(format!("{:#?}", response), "SetxattrResponse",);
}
