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
use fuse::operations::mknod::{MknodRequest, MknodResponse};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

const S_IFBLK: u32 = 0o60000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&0o644u32)
		.push_sized(&0u32)
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MknodRequest, buf, {
		protocol_version: (7, 1),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.parent_id(), node::Id::new(100).unwrap());
	assert_eq!(req.name(), expect);
	assert_eq!(req.mode(), node::Mode::new(0o644));
	assert_eq!(req.umask(), 0);
	assert_eq!(req.device_number(), None);
}

#[test]
fn request_v7p12() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&fuse_kernel::fuse_mknod_in {
			mode: 0o644,
			rdev: 0,
			umask: 0o111,
			padding: 0,
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MknodRequest, buf, {
		protocol_version: (7, 12),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.parent_id(), node::Id::new(100).unwrap());
	assert_eq!(req.name(), expect);
	assert_eq!(req.mode(), node::Mode::new(0o644));
	assert_eq!(req.umask(), 0o111);
	assert_eq!(req.device_number(), None);
}

#[test]
fn request_device_number() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&fuse_kernel::fuse_mknod_in {
			mode: S_IFBLK | 0o644,
			rdev: 123,
			umask: 0o111,
			padding: 0,
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MknodRequest, buf, {
		protocol_version: (7, 12),
	});

	assert_eq!(node::Type::from_mode(req.mode()), Some(node::Type::BlockDevice));
	assert_eq!(req.mode().permissions(), 0o644);
	assert_eq!(req.device_number(), Some(123));
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&fuse_kernel::fuse_mknod_in {
			mode: S_IFBLK | 0o644,
			rdev: 123,
			umask: 0o111,
			padding: 0,
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let request = decode_request!(MknodRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"MknodRequest {\n",
			"    parent_id: 100,\n",
			"    name: \"hello.world!\",\n",
			"    mode: 0o60644,\n",
			"    umask: 0o111,\n",
			"    device_number: Some(123),\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let attr = node::Attributes::new(node::Id::new(11).unwrap());
	let mut entry = node::Entry::new(attr);
	entry.set_generation(22);
	let resp = MknodResponse::new(entry);

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
	let attr = node::Attributes::new(node::Id::new(11).unwrap());
	let mut entry = node::Entry::new(attr);
	entry.set_generation(22);
	let resp = MknodResponse::new(entry);

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
	let mut attr = node::Attributes::new(node::Id::new(11).unwrap());
	attr.set_mode(node::Mode::S_IFREG | 0o644);
	let mut entry = node::Entry::new(attr);
	entry.set_generation(22);
	let response = MknodResponse::new(entry);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"MknodResponse {\n",
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
