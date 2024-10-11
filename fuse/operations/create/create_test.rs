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
use fuse::server::CreateRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_CREATE;
			h.nodeid = 123;
		})
		.push_sized(&0xFFu32) // fuse_create_in::flags
		.push_sized(&0u32) // fuse_create_in::unused
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(CreateRequest, buf, {
		protocol_version: (7, 1),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
	assert_eq!(req.flags(), fuse::CreateRequestFlags::new());
	assert_eq!(req.open_flags(), 0xFF);
	assert_eq!(req.mode(), fuse::FileMode::new(0));
	assert_eq!(req.umask(), 0);
}

#[test]
fn request_v7p12() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_CREATE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_create_in {
			flags: 0xFF,
			mode: 0xEE,
			umask: 0xDD,
			open_flags: 0, // TODO
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(CreateRequest, buf, {
		protocol_version: (7, 12),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
	assert_eq!(req.flags(), fuse::CreateRequestFlags::new());
	assert_eq!(req.open_flags(), 0xFF);
	assert_eq!(req.mode(), fuse::FileMode::new(0xEE));
	assert_eq!(req.umask(), 0xDD);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, CreateRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_CREATE;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_create_in {
			flags: 123,
			mode: 0o100644,
			umask: 0o22,
			open_flags: 0, // TODO
		}))
		.push_bytes(b"hello.world\x00")
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"CreateRequest {\n",
			"    node_id: 1,\n",
			"    name: \"hello.world\",\n",
			"    flags: CreateRequestFlags {},\n",
			"    open_flags: 0x0000007B,\n",
			"    mode: 0o100644,\n",
			"    umask: 18,\n",
			"}",
		),
	);
}
