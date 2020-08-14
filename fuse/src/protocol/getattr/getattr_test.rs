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
use crate::protocol::node;
use crate::protocol::prelude::*;

use super::{GetattrRequest, GetattrResponse};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_GETATTR;
			h.nodeid = 123;
		})
		.build_aligned();

	let req: GetattrRequest = decode_request!(buf, {
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

	let req: GetattrRequest = decode_request!(buf, {
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

	let req: GetattrRequest = decode_request!(buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), Some(123));
}

#[test]
fn request_impl_debug() {
	let request = &GetattrRequest {
		phantom: PhantomData,
		node_id: node::NodeId::ROOT,
		handle: Some(123),
	};

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
	let node_id = node::NodeId::new(0xABCD).unwrap();
	let mut resp = GetattrResponse::new();
	resp.attr_mut().set_node_id(node_id);
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
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 0,
				attr_valid_nsec: 0,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: node_id.get(),
					..Default::default()
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
	let node_id = node::NodeId::new(0xABCD).unwrap();
	let mut resp = GetattrResponse::new();
	resp.attr_mut().set_node_id(node_id);
	resp.attr_mut().set_size(999);
	resp.set_attr_timeout(time::Duration::new(123, 456));

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
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 123,
				attr_valid_nsec: 456,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					ino: node_id.get(),
					size: 999,
					..Default::default()
				},
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let node_id = node::NodeId::new(11).unwrap();
	let mut response = GetattrResponse::new();
	response.attr_mut().set_node_id(node_id);
	response.attr_mut().set_size(999);
	response.attr_mut().set_mode(node::NodeKind::REG | 0o644);
	response.set_attr_timeout(time::Duration::new(123, 456));

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"GetattrResponse {\n",
			"    attr_timeout: 123.000000456s,\n",
			"    attr: NodeAttr {\n",
			"        node_id: Some(11),\n",
			"        size: 999,\n",
			"        blocks: 0,\n",
			"        atime: 0ns,\n",
			"        mtime: 0ns,\n",
			"        ctime: 0ns,\n",
			"        mode: 0o100644,\n",
			"        nlink: 0,\n",
			"        uid: 0,\n",
			"        gid: 0,\n",
			"        rdev: 0,\n",
			"        blksize: 0,\n",
			"    },\n",
			"}",
		),
	);
}
