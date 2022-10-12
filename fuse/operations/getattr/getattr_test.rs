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
use core::time;

use fuse::node;
use fuse::operations::getattr::{GetattrRequest, GetattrResponse};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETATTR;
			h.nodeid = 123;
		})
		.build_aligned();

	let req = decode_request!(GetattrRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), None);
}

#[test]
fn request_v7p9() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getattr_in {
			getattr_flags: 0,
			dummy: 0,
			fh: 0,
		})
		.build_aligned();

	let req = decode_request!(GetattrRequest, buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), None);
}

#[test]
fn request_v7p9_with_handle() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETATTR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_getattr_in {
			getattr_flags: fuse_kernel::FUSE_GETATTR_FH,
			dummy: 0,
			fh: 123,
		})
		.build_aligned();

	let req = decode_request!(GetattrRequest, buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), Some(123));
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, GetattrRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETATTR;
			h.nodeid = fuse_kernel::FUSE_ROOT_ID;
		})
		.push_sized(&fuse_kernel::fuse_getattr_in {
			getattr_flags: fuse_kernel::FUSE_GETATTR_FH,
			dummy: 0,
			fh: 123,
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"GetattrRequest {\n",
			"    node_id: 1,\n",
			"    handle: Some(123),\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let node_id = node::Id::new(0xABCD).unwrap();
	let attr = node::Attributes::new(node_id);
	let resp = GetattrResponse::new(attr);
	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 0,
				attr_valid_nsec: 0,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: node_id.get(),
					..fuse_kernel::fuse_attr::zeroed()
				},
			})
			.unpush(
				size_of::<fuse_kernel::fuse_attr_out>()
					- fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE
			)
			.build()
	);
}

#[test]
fn response_v7p9() {
	let node_id = node::Id::new(0xABCD).unwrap();
	let mut attr = node::Attributes::new(node_id);
	attr.set_size(999);

	let mut resp = GetattrResponse::new(attr);
	resp.set_cache_timeout(time::Duration::new(123, 456));

	let encoded = encode_response!(resp, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_attr_out>()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 123,
				attr_valid_nsec: 456,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: node_id.get(),
					size: 999,
					..fuse_kernel::fuse_attr::zeroed()
				},
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let node_id = node::Id::new(11).unwrap();

	let mut attr = node::Attributes::new(node_id);
	attr.set_size(999);
	attr.set_mode(node::Mode::S_IFREG | 0o644);

	let mut response = GetattrResponse::new(attr);
	response.set_cache_timeout(time::Duration::new(123, 456));

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"GetattrResponse {\n",
			"    attributes: Attributes {\n",
			"        node_id: 11,\n",
			"        mode: 0o100644,\n",
			"        size: 999,\n",
			"        atime: UnixTime(0.000000000),\n",
			"        mtime: UnixTime(0.000000000),\n",
			"        ctime: UnixTime(0.000000000),\n",
			"        link_count: 0,\n",
			"        user_id: 0,\n",
			"        group_id: 0,\n",
			"        device_number: 0,\n",
			"        block_count: 0,\n",
			"        block_size: 0,\n",
			"        flags: AttributeFlags {},\n",
			"    },\n",
			"    cache_timeout: 123.000000456s,\n",
			"}",
		),
	);
}
