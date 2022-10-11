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

#[cfg(target_os = "linux")]
use linux_errno as os_errno;

#[cfg(target_os = "freebsd")]
use freebsd_errno as os_errno;

use fuse::node;
use fuse::operations::lookup::{LookupRequest, LookupResponse};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LOOKUP;
			h.nodeid = 123;
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let req = decode_request!(LookupRequest, buf);

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LOOKUP;
			h.nodeid = 123;
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let request = decode_request!(LookupRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"LookupRequest {\n",
			"    parent_id: 123,\n",
			"    name: \"hello.world!\",\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let mut response = LookupResponse::new();
	let node = response.node_mut();
	node.set_id(node::Id::new(11).unwrap());
	node.set_generation(22);
	node.attr_mut().set_node_id(node::Id::new(11).unwrap());
	node.attr_mut().set_mode(node::Mode::S_IFREG | 0o644);

	let encoded = encode_response!(response, {
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
					mode: 0o100644,
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
	let mut response = LookupResponse::new();
	let node = response.node_mut();
	node.set_id(node::Id::new(11).unwrap());
	node.set_generation(22);
	node.attr_mut().set_node_id(node::Id::new(11).unwrap());
	node.attr_mut().set_mode(node::Mode::S_IFREG | 0o644);

	let encoded = encode_response!(response, {
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
					mode: 0o100644,
					..fuse_kernel::fuse_attr::zeroed()
				}
			})
			.build()
	);
}

#[test]
fn response_noexist_v7p1() {
	let resp = LookupResponse::new();
	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: size_of::<fuse_kernel::fuse_out_header>() as u32,
				error: -(os_errno::ENOENT.get() as i32),
				unique: 0,
			})
			.build()
	);
}

#[test]
fn response_noexist_v7p4() {
	let resp = LookupResponse::new();
	let encoded = encode_response!(resp, {
		protocol_version: (7, 4),
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
				nodeid: 0,
				generation: 0,
				entry_valid: 0,
				attr_valid: 0,
				entry_valid_nsec: 0,
				attr_valid_nsec: 0,
				attr: fuse_kernel::fuse_attr::zeroed(),
			})
			.unpush(
				size_of::<fuse_kernel::fuse_entry_out>()
					- fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE
			)
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = LookupResponse::new();
	let node = response.node_mut();
	node.set_id(node::Id::new(11).unwrap());
	node.set_generation(22);
	node.attr_mut().set_node_id(node::Id::new(11).unwrap());
	node.attr_mut().set_mode(node::Mode::S_IFREG | 0o644);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"LookupResponse {\n",
			"    node: Node {\n",
			"        id: Some(11),\n",
			"        generation: 22,\n",
			"        cache_timeout: 0ns,\n",
			"        attr_cache_timeout: 0ns,\n",
			"        attr: NodeAttr {\n",
			"            node_id: Some(11),\n",
			"            size: 0,\n",
			"            blocks: 0,\n",
			"            atime: UnixTime(0.000000000),\n",
			"            mtime: UnixTime(0.000000000),\n",
			"            ctime: UnixTime(0.000000000),\n",
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
