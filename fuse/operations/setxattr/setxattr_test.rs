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

use crate::XattrName;
use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{SetxattrRequest, SetxattrResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_SETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&super::fuse_setxattr_in_v7p1 {
			size: 10,
			flags: 0b11,
		})
		.push_bytes(b"hello.world!\x00")
		.push_bytes(b"some\x00value")
		.build_aligned();

	let req = decode_request!(SetxattrRequest, buf);

	let expect_name = XattrName::from_bytes(b"hello.world!").unwrap();
	assert_eq!(req.name(), expect_name);
	assert_eq!(req.value(), b"some\x00value");
	assert_eq!(req.flags().create, true);
	assert_eq!(req.flags().replace, true);
}

#[test]
fn request_impl_debug() {
	let request = &SetxattrRequest {
		node_id: crate::ROOT_ID,
		name: XattrName::from_bytes(b"hello.world!").unwrap(),
		flags: super::SetxattrRequestFlags::from_bits(0),
		value: b"some\x00value",
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"SetxattrRequest {\n",
			"    node_id: 1,\n",
			"    name: \"hello.world!\",\n",
			"    flags: SetxattrRequestFlags {\n",
			"        create: false,\n",
			"        replace: false,\n",
			"    },\n",
			"    value: \"some\\x00value\",\n",
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
			.push_sized(&fuse_kernel::fuse_out_header {
				len: size_of::<fuse_kernel::fuse_out_header>() as u32,
				error: 0,
				unique: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let response = SetxattrResponse::new();

	assert_eq!(format!("{:#?}", response), "SetxattrResponse",);
}
