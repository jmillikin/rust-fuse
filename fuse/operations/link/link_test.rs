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
use fuse::operations::link::{LinkRequest, LinkResponse};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LINK;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_link_in {
			oldnodeid: 123,
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(LinkRequest, buf);

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.node_id(), fuse::NodeId::new(123).unwrap());
	assert_eq!(req.new_parent_id(), fuse::NodeId::new(100).unwrap());
	assert_eq!(req.new_name(), expect.as_ref());
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LINK;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_link_in {
			oldnodeid: 123,
		}))
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
	let attr = fuse::Attributes::new(fuse::NodeId::new(11).unwrap());
	let mut entry = fuse::Entry::new(attr);
	entry.set_generation(22);
	let resp = LinkResponse::new(entry);

	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_out_header {
				len: (size_of::<kernel::fuse_out_header>()
					+ kernel::FUSE_COMPAT_ENTRY_OUT_SIZE) as u32,
				unique: 0xAABBCCDD,
			}))
			.push_sized(&testutil::new!(kernel::fuse_entry_out {
				nodeid: 11,
				generation: 22,
				attr: testutil::new!(kernel::fuse_attr {
					ino: 11,
				}),
			}))
			.unpush(
				size_of::<kernel::fuse_entry_out>()
					- kernel::FUSE_COMPAT_ENTRY_OUT_SIZE
			)
			.build()
	);
}

#[test]
fn response_v7p9() {
	let attr = fuse::Attributes::new(fuse::NodeId::new(11).unwrap());
	let mut entry = fuse::Entry::new(attr);
	entry.set_generation(22);
	let resp = LinkResponse::new(entry);

	let encoded = encode_response!(resp, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_out_header {
				len: (size_of::<kernel::fuse_out_header>()
					+ size_of::<kernel::fuse_entry_out>()) as u32,
				unique: 0xAABBCCDD,
			}))
			.push_sized(&testutil::new!(kernel::fuse_entry_out {
				nodeid: 11,
				generation: 22,
				attr: testutil::new!(kernel::fuse_attr {
					ino: 11,
				}),
			}))
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut attr = fuse::Attributes::new(fuse::NodeId::new(11).unwrap());
	attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
	let mut entry = fuse::Entry::new(attr);
	entry.set_generation(22);
	let response = LinkResponse::new(entry);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"LinkResponse {\n",
			"    entry: Entry {\n",
			"        generation: 22,\n",
			"        attributes: Attributes {\n",
			"            node_id: 11,\n",
			"            mode: 0o100644,\n",
			"            size: 0,\n",
			"            atime: UnixTime(0.000000000),\n",
			"            mtime: UnixTime(0.000000000),\n",
			"            ctime: UnixTime(0.000000000),\n",
			"            link_count: 0,\n",
			"            user_id: 0,\n",
			"            group_id: 0,\n",
			"            device_number: 0,\n",
			"            block_count: 0,\n",
			"            block_size: 0,\n",
			"            flags: AttributeFlags {},\n",
			"        },\n",
			"        cache_timeout: 0ns,\n",
			"        attribute_cache_timeout: 0ns,\n",
			"    },\n",
			"}",
		),
	);
}
