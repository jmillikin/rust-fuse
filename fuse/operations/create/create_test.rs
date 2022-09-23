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

use crate::FileType;
use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{CreateRequest, CreateResponse};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_CREATE;
			h.nodeid = 123;
		})
		.push_sized(&super::fuse_create_in_v7p1 {
			flags: 0xFF,
			unused: 0,
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(CreateRequest, buf, {
		protocol_version: (7, 1),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
	assert_eq!(req.flags(), 0xFF);
	assert_eq!(req.mode(), 0);
	assert_eq!(req.umask(), 0);
}

#[test]
fn request_v7p12() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_CREATE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_create_in {
			flags: 0xFF,
			mode: 0xEE,
			umask: 0xDD,
			open_flags: 0, // TODO
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(CreateRequest, buf, {
		protocol_version: (7, 12),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
	assert_eq!(req.flags(), 0xFF);
	assert_eq!(req.mode(), 0xEE);
	assert_eq!(req.umask(), 0xDD);
}

#[test]
fn request_impl_debug() {
	let request = &CreateRequest {
		node_id: crate::ROOT_ID,
		name: NodeName::from_bytes(b"hello.world").unwrap(),
		flags: 123,
		mode: 0o100644,
		umask: 0o22,
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"CreateRequest {\n",
			"    node_id: 1,\n",
			"    name: \"hello.world\",\n",
			"    flags: 123,\n",
			"    mode: 0o100644,\n",
			"    umask: 18,\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let mut resp = CreateResponse::new();
	resp.node_mut().set_id(NodeId::new(11).unwrap());
	resp.node_mut().set_generation(22);
	resp.node_mut()
		.attr_mut()
		.set_node_id(NodeId::new(11).unwrap());
	resp.set_handle(123);
	resp.flags_mut().direct_io = true;
	resp.flags_mut().keep_cache = true;

	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE
					+ size_of::<fuse_kernel::fuse_open_out>()) as u32,
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
			.push_sized(&fuse_kernel::fuse_open_out {
				fh: 123,
				open_flags: 0b11,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_v7p9() {
	let mut resp = CreateResponse::new();
	resp.node_mut().set_id(NodeId::new(11).unwrap());
	resp.node_mut().set_generation(22);
	resp.node_mut()
		.attr_mut()
		.set_node_id(NodeId::new(11).unwrap());
	resp.set_handle(123);
	resp.flags_mut().direct_io = true;
	resp.flags_mut().keep_cache = true;

	let encoded = encode_response!(resp, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_entry_out>()
					+ size_of::<fuse_kernel::fuse_open_out>()) as u32,
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
			.push_sized(&fuse_kernel::fuse_open_out {
				fh: 123,
				open_flags: 0b11,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = CreateResponse::new();

	let node = response.node_mut();
	node.set_id(NodeId::new(11).unwrap());
	node.set_generation(22);
	node.attr_mut().set_node_id(NodeId::new(11).unwrap());
	node.attr_mut().set_mode(FileType::Regular | 0o644);

	response.set_handle(123);
	response.flags_mut().direct_io = true;
	response.flags_mut().keep_cache = true;

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"CreateResponse {\n",
			"    node: fuse_entry_out {\n",
			"        nodeid: 11,\n",
			"        generation: 22,\n",
			"        entry_valid: 0,\n",
			"        attr_valid: 0,\n",
			"        entry_valid_nsec: 0,\n",
			"        attr_valid_nsec: 0,\n",
			"        attr: fuse_attr {\n",
			"            ino: 11,\n",
			"            size: 0,\n",
			"            blocks: 0,\n",
			"            atime: 0,\n",
			"            mtime: 0,\n",
			"            ctime: 0,\n",
			"            atimensec: 0,\n",
			"            mtimensec: 0,\n",
			"            ctimensec: 0,\n",
			"            mode: 33188,\n",
			"            nlink: 0,\n",
			"            uid: 0,\n",
			"            gid: 0,\n",
			"            rdev: 0,\n",
			"            blksize: 0,\n",
			"            flags: 0,\n",
			"        },\n",
			"    },\n",
			"    handle: 123,\n",
			"    flags: CreateResponseFlags {\n",
			"        direct_io: true,\n",
			"        keep_cache: true,\n",
			"        nonseekable: false,\n",
			"    },\n",
			"}",
		),
	);
}
