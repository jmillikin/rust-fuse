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

use core::marker::PhantomData;
use core::mem::size_of;
use core::num;

use crate::XattrName;
use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{ListxattrRequest, ListxattrResponse};

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

	assert_eq!(req.size(), Some(num::NonZeroU32::new(10).unwrap()));
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
	let request = &ListxattrRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
		size: num::NonZeroU32::new(11),
	};

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
fn response_sized_heap() {
	let mut resp = ListxattrResponse::with_max_size(10);
	response_sized_test_impl(&mut resp);
}

#[test]
fn response_sized_stack() {
	let mut buf = [0u8; 10];
	let mut resp = ListxattrResponse::with_capacity(&mut buf);
	response_sized_test_impl(&mut resp);
}

fn response_sized_test_impl(resp: &mut ListxattrResponse) {
	// response must fit in kernel buffer
	{
		let name = XattrName::from_bytes(b"12345678901").unwrap();
		assert!(resp.try_add_name(name).is_err());
	}

	// xattr names are NUL-terminated
	resp.add_name(XattrName::from_bytes(b"123").unwrap());
	resp.add_name(XattrName::from_bytes(b"456").unwrap());

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
fn response_without_capacity() {
	let mut resp = ListxattrResponse::without_capacity();

	// set_value() doesn't store value bytes for responses without capacity
	resp.add_name(XattrName::from_bytes(b"123").unwrap());
	resp.add_name(XattrName::from_bytes(b"456").unwrap());

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
				size: 8,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_size_limit() {
	// listxattr response size can't exceed XATTR_LIST_MAX
	let mut resp = ListxattrResponse::without_capacity();
	let name = XattrName::from_bytes(&[b'a'; 250]).unwrap();
	for _ in 0..261 {
		resp.add_name(name);
	}
	assert!(resp.try_add_name(name).is_err());
}

#[test]
fn response_sized_impl_debug() {
	let mut response = ListxattrResponse::with_max_size(10);

	response.add_name(XattrName::from_bytes(b"123").unwrap());
	response.add_name(XattrName::from_bytes(b"456").unwrap());

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
fn response_without_capacity_impl_debug() {
	let mut response = ListxattrResponse::without_capacity();

	response.add_name(XattrName::from_bytes(b"123").unwrap());
	response.add_name(XattrName::from_bytes(b"456").unwrap());

	assert_eq!(
		format!("{:#?}", response),
		concat!("ListxattrResponse {\n", "    size: 8,\n", "}",),
	);
}
