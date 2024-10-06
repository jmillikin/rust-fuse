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
use fuse::operations::mkdir::{MkdirRequest, MkdirResponse};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_MKDIR;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_mkdir_in {
			mode: 0o755,
			umask: 0o111,
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MkdirRequest, buf);

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.parent_id(), fuse::NodeId::new(100).unwrap());
	assert_eq!(req.name(), expect);
	assert_eq!(req.mode(), fuse::FileMode::new(0o755));
	assert_eq!(req.umask(), 0o111);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_MKDIR;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_mkdir_in {
			mode: 0o755,
			umask: 0o111,
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let request = decode_request!(MkdirRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"MkdirRequest {\n",
			"    parent_id: 100,\n",
			"    name: \"hello.world!\",\n",
			"    mode: 0o755,\n",
			"    umask: 0o111,\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let attr = fuse::Attributes::new(fuse::NodeId::new(11).unwrap());
	let mut entry = fuse::Entry::new(attr);
	entry.set_generation(22);
	let resp = MkdirResponse::new(entry);

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
	let resp = MkdirResponse::new(entry);

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
	let response = MkdirResponse::new(entry);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"MkdirResponse {\n",
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
