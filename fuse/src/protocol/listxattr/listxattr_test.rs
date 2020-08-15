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
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LISTXATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getxattr_in {
			size: 10,
			..Default::default()
		})
		.build_aligned();

	let req: ListxattrRequest = decode_request!(buf);

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
			..Default::default()
		})
		.build_aligned();

	let req: ListxattrRequest = decode_request!(buf);

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
	let request_size = num::NonZeroU32::new(10);
	let mut resp = ListxattrResponse::new(request_size);
	response_sized_test_impl(&mut resp);
}

#[test]
fn response_sized_stack() {
	let request_size = num::NonZeroU32::new(10);
	let mut buf = [0u8; 1024];
	let mut resp = ListxattrResponse::with_capacity(request_size, &mut buf);
	response_sized_test_impl(&mut resp);
}

fn response_sized_test_impl(resp: &mut ListxattrResponse) {
	assert_eq!(resp.request_size(), num::NonZeroU32::new(10));

	// response must fit in kernel buffer
	{
		let name = XattrName::from_bytes(b"12345678901").unwrap();
		assert!(resp.try_add_name(name).is_none());
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
fn response_unsized() {
	let mut resp = ListxattrResponse::new(None);
	assert_eq!(resp.request_size(), None);

	// set_value() doesn't store value bytes for unsized responses
	resp.add_name(XattrName::from_bytes(b"123").unwrap());
	assert_eq!(resp.raw.size, 4);

	resp.add_name(XattrName::from_bytes(b"456").unwrap());
	assert_eq!(resp.raw.size, 8);

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
	let mut resp = ListxattrResponse::new(None);
	let name = XattrName::from_bytes(&[b'a'; 250]).unwrap();
	for _ in 0..261 {
		resp.add_name(name);
	}
	assert_eq!(resp.raw.size, 65511);
	assert!(resp.try_add_name(name).is_none());
}

#[test]
fn response_sized_impl_debug() {
	let request_size = num::NonZeroU32::new(10);
	let mut response = ListxattrResponse::new(request_size);

	response.add_name(XattrName::from_bytes(b"123").unwrap());
	response.add_name(XattrName::from_bytes(b"456").unwrap());

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ListxattrResponse {\n",
			"    request_size: Some(10),\n",
			"    size: 8,\n",
			"    names: [\n",
			"        \"123\",\n",
			"        \"456\",\n",
			"    ],\n",
			"}",
		),
	);
}

#[test]
fn response_unsized_impl_debug() {
	let mut response = ListxattrResponse::new(None);

	response.add_name(XattrName::from_bytes(b"123").unwrap());
	response.add_name(XattrName::from_bytes(b"456").unwrap());

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ListxattrResponse {\n",
			"    request_size: None,\n",
			"    size: 8,\n",
			"    names: [],\n",
			"}",
		),
	);
}
