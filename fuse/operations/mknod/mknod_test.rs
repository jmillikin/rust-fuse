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

use fuse::kernel;
use fuse::server::MknodRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

const S_IFBLK: u32 = 0o60000;

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&0o644u32)
		.push_sized(&0u32)
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MknodRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.parent_id(), fuse::NodeId::new(100).unwrap());
	assert_eq!(req.name(), "hello.world!");
	assert_eq!(req.mode(), fuse::FileMode::new(0o644));
	assert_eq!(req.umask(), 0);
	assert_eq!(req.device_number(), None);
}

#[test]
fn request_v7p12() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_mknod_in {
			mode: 0o644,
			umask: 0o111,
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MknodRequest, buf, {
		protocol_version: (7, 12),
	});

	assert_eq!(req.parent_id(), fuse::NodeId::new(100).unwrap());
	assert_eq!(req.name(), "hello.world!");
	assert_eq!(req.mode(), fuse::FileMode::new(0o644));
	assert_eq!(req.umask(), 0o111);
	assert_eq!(req.device_number(), None);
}

#[test]
fn request_device_number() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_mknod_in {
			mode: S_IFBLK | 0o644,
			rdev: 123,
			umask: 0o111,
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(MknodRequest, buf, {
		protocol_version: (7, 12),
	});

	assert_eq!(
		fuse::FileType::from_mode(req.mode()),
		Some(fuse::FileType::BlockDevice)
	);
	assert_eq!(req.mode().permissions(), 0o644);
	assert_eq!(req.device_number(), Some(123));
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_MKNOD;
			h.nodeid = 100;
		})
		.push_sized(&testutil::new!(kernel::fuse_mknod_in {
			mode: S_IFBLK | 0o644,
			rdev: 123,
			umask: 0o111,
		}))
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
