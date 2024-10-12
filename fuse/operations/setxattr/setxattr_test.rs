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
use fuse::server::SetxattrRequest;

use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&10u32) // fuse_setxattr_in::size
		.push_sized(&0b11u32) // fuse_setxattr_in::flags
		.push_bytes(b"hello.world!\x00")
		.push_bytes(b"some\x00value")
		.build_aligned();

	let req = decode_request!(SetxattrRequest, buf);

	assert_eq!(req.name(), c"hello.world!");
	assert_eq!(req.value(), b"some\x00value");
	assert_eq!(req.setxattr_flags(), 0b11);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, SetxattrRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SETXATTR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&10u32) // fuse_setxattr_in::size
		.push_sized(&0u32) // fuse_setxattr_in::flags
		.push_bytes(b"hello.world!\x00")
		.push_bytes(b"some\x00value")
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"SetxattrRequest {\n",
			"    node_id: 1,\n",
			"    name: \"hello.world!\",\n",
			"    flags: SetxattrRequestFlags {},\n",
			"    setxattr_flags: 0x00000000,\n",
			"    value: \"some\\x00value\",\n",
			"}",
		),
	);
}
