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

use fuse::node;
use fuse::operations::link::{LinkRequest, LinkResponse};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LINK;
			h.nodeid = 100;
		})
		.push_sized(&fuse_kernel::fuse_link_in { oldnodeid: 123 })
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(LinkRequest, buf);

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.node_id(), node::Id::new(123).unwrap());
	assert_eq!(req.new_parent_id(), node::Id::new(100).unwrap());
	assert_eq!(req.new_name(), expect.as_ref());
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LINK;
			h.nodeid = 100;
		})
		.push_sized(&fuse_kernel::fuse_link_in { oldnodeid: 123 })
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let request = decode_request!(LinkRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"LinkRequest {\n",
			"    node_id: 123,\n",
			"    new_parent_id: 100,\n",
			"    new_name: \"hello.world!\",\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let mut resp = LinkResponse::new();
	resp.node_mut().set_id(node::Id::new(11).unwrap());
	resp.node_mut().set_generation(22);
	resp.node_mut()
		.attr_mut()
		.set_node_id(node::Id::new(11).unwrap());

	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_entry_out {
				nodeid: 11,
				generation: 22,
				entry_valid: 0,
				attr_valid: 0,
				entry_valid_nsec: 0,
				attr_valid_nsec: 0,
				attr: fuse_kernel::fuse_attr {
					ino: 11,
					..fuse_kernel::fuse_attr::zeroed()
				}
			})
			.unpush(
				size_of::<fuse_kernel::fuse_entry_out>()
					- fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE
			)
			.build()
	);
}

#[test]
fn response_v7p9() {
	let mut resp = LinkResponse::new();
	resp.node_mut().set_id(node::Id::new(11).unwrap());
	resp.node_mut().set_generation(22);
	resp.node_mut()
		.attr_mut()
		.set_node_id(node::Id::new(11).unwrap());

	let encoded = encode_response!(resp, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_entry_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_entry_out {
				nodeid: 11,
				generation: 22,
				entry_valid: 0,
				attr_valid: 0,
				entry_valid_nsec: 0,
				attr_valid_nsec: 0,
				attr: fuse_kernel::fuse_attr {
					ino: 11,
					..fuse_kernel::fuse_attr::zeroed()
				}
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = LinkResponse::new();
	let node = response.node_mut();
	node.set_id(node::Id::new(11).unwrap());
	node.set_generation(22);
	node.attr_mut().set_node_id(node::Id::new(11).unwrap());
	node.attr_mut().set_mode(node::Mode::S_IFREG | 0o644);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"LinkResponse {\n",
			"    node: Node {\n",
			"        id: Some(11),\n",
			"        generation: 22,\n",
			"        cache_timeout: 0ns,\n",
			"        attr_cache_timeout: 0ns,\n",
			"        attr: NodeAttr {\n",
			"            node_id: Some(11),\n",
			"            size: 0,\n",
			"            blocks: 0,\n",
			"            atime: 0ns,\n",
			"            mtime: 0ns,\n",
			"            ctime: 0ns,\n",
			"            mode: 0o100644,\n",
			"            nlink: 0,\n",
			"            uid: 0,\n",
			"            gid: 0,\n",
			"            rdev: 0,\n",
			"            blksize: 0,\n",
			"        },\n",
			"    },\n",
			"}",
		),
	);
}
