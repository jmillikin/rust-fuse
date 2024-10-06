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
use fuse::operations::statfs::{
	StatfsAttributes,
	StatfsRequest,
	StatfsResponse,
};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_STATFS;
			h.nodeid = 123;
		})
		.build_aligned();

	let _req = decode_request!(StatfsRequest, buf);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, StatfsRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_STATFS;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!("StatfsRequest {\n", "    node_id: 1,\n", "}",),
	);
}

#[test]
fn response_v7p1() {
	let mut attr = StatfsAttributes::new();
	attr.set_block_count(10);
	attr.set_block_size(20);
	attr.set_blocks_available(30);
	attr.set_blocks_free(40);
	attr.set_fragment_size(50);
	attr.set_inode_count(60);
	attr.set_inodes_free(70);
	attr.set_max_filename_length(80);

	let response = StatfsResponse::new(attr);
	let encoded = encode_response!(response, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_out_header {
				len: (size_of::<kernel::fuse_out_header>()
					+ kernel::FUSE_COMPAT_STATFS_SIZE) as u32,
				unique: 0xAABBCCDD,
			}))
			.push_sized(&testutil::new!(kernel::fuse_statfs_out {
				st: testutil::new!(kernel::fuse_kstatfs {
					blocks: 10,
					bsize: 20,
					bavail: 30,
					bfree: 40,
					frsize: 50,
					files: 60,
					ffree: 70,
					namelen: 80,
				}),
			}))
			.unpush(
				size_of::<kernel::fuse_statfs_out>()
					- kernel::FUSE_COMPAT_STATFS_SIZE
			)
			.build()
	);
}

#[test]
fn response_v7p4() {
	let mut attr = StatfsAttributes::new();
	attr.set_block_count(10);
	attr.set_block_size(20);
	attr.set_blocks_available(30);
	attr.set_blocks_free(40);
	attr.set_fragment_size(50);
	attr.set_inode_count(60);
	attr.set_inodes_free(70);
	attr.set_max_filename_length(80);

	let response = StatfsResponse::new(attr);
	let encoded = encode_response!(response, {
		protocol_version: (7, 4),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_out_header {
				len: (size_of::<kernel::fuse_out_header>()
					+ size_of::<kernel::fuse_statfs_out>()) as u32,
				unique: 0xAABBCCDD,
			}))
			.push_sized(&testutil::new!(kernel::fuse_statfs_out {
				st: testutil::new!(kernel::fuse_kstatfs {
					blocks: 10,
					bsize: 20,
					bavail: 30,
					bfree: 40,
					frsize: 50,
					files: 60,
					ffree: 70,
					namelen: 80,
				}),
			}))
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut attr = StatfsAttributes::new();
	attr.set_block_count(10);
	attr.set_block_size(20);
	attr.set_blocks_available(30);
	attr.set_blocks_free(40);
	attr.set_fragment_size(50);
	attr.set_inode_count(60);
	attr.set_inodes_free(70);
	attr.set_max_filename_length(80);
	let response = StatfsResponse::new(attr);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"StatfsResponse {\n",
			"    attributes: StatfsAttributes {\n",
			"        block_count: 10,\n",
			"        block_size: 20,\n",
			"        blocks_available: 30,\n",
			"        blocks_free: 40,\n",
			"        fragment_size: 50,\n",
			"        inode_count: 60,\n",
			"        inodes_free: 70,\n",
			"        max_filename_length: 80,\n",
			"    },\n",
			"}",
		),
	);
}
