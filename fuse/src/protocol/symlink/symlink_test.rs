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

use super::{SymlinkRequest, SymlinkResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_SYMLINK;
			h.nodeid = 100;
		})
		.push_bytes(b"link content\x00")
		.push_bytes(b"link name\x00")
		.build_aligned();
	let request: SymlinkRequest = decode_request!(buf);

	let expect_content: &[u8] = b"link content";
	let expect_name: &[u8] = b"link name";
	assert_eq!(request.parent_id(), NodeId::new(100).unwrap());
	assert_eq!(request.name(), expect_name);
	assert_eq!(request.content(), expect_content);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_SYMLINK;
			h.nodeid = 100;
		})
		.push_bytes(b"link content\x00")
		.push_bytes(b"link name\x00")
		.build_aligned();
	let request: SymlinkRequest = decode_request!(buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"SymlinkRequest {\n",
			"    parent_id: 100,\n",
			"    name: \"link name\",\n",
			"    content: \"link content\",\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let mut resp = SymlinkResponse::new();
	resp.node_mut().set_id(NodeId::new(11).unwrap());
	resp.node_mut().set_generation(22);
	resp.node_mut()
		.attr_mut()
		.set_node_id(NodeId::new(11).unwrap());

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
	let mut resp = SymlinkResponse::new();
	resp.node_mut().set_id(NodeId::new(11).unwrap());
	resp.node_mut().set_generation(22);
	resp.node_mut()
		.attr_mut()
		.set_node_id(NodeId::new(11).unwrap());

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
	let mut response = SymlinkResponse::new();
	let node = response.node_mut();
	node.set_id(NodeId::new(11).unwrap());
	node.set_generation(22);
	node.attr_mut().set_node_id(NodeId::new(11).unwrap());
	node.attr_mut().set_mode(FileType::Regular | 0o644);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"SymlinkResponse {\n",
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
