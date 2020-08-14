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

use super::{MknodRequest, MknodResponse};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_MKNOD)
		.push_sized(&super::fuse_mknod_in_v7p1 { mode: 11, rdev: 22 })
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req: MknodRequest = decode_request!(buf, {
		protocol_version: (7, 1),
	});

	let expect = CString::new("hello.world!").unwrap();
	assert_eq!(req.name(), expect.as_ref());
	assert_eq!(req.mode(), 11);
	assert_eq!(req.rdev(), 22);
	assert_eq!(req.umask(), 0);
}

#[test]
fn request_v7p12() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_MKNOD)
		.push_sized(&fuse_kernel::fuse_mknod_in {
			mode: 11,
			rdev: 22,
			umask: 33,
			padding: 0,
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req: MknodRequest = decode_request!(buf, {
		protocol_version: (7, 12),
	});

	let expect = CString::new("hello.world!").unwrap();
	assert_eq!(req.name(), expect.as_ref());
	assert_eq!(req.mode(), 11);
	assert_eq!(req.rdev(), 22);
	assert_eq!(req.umask(), 33);
}

#[test]
fn response_v7p1() {
	let mut resp = MknodResponse::new();
	resp.node_mut().set_node_id(node::NodeId::new(11).unwrap());
	resp.node_mut().set_generation(22);

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
					..Default::default()
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
	let mut resp = MknodResponse::new();
	resp.node_mut().set_node_id(node::NodeId::new(11).unwrap());
	resp.node_mut().set_generation(22);

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
					..Default::default()
				}
			})
			.build()
	);
}
