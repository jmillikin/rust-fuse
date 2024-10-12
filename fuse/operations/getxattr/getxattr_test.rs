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

use core::num;

use fuse::kernel;
use fuse::server::GetxattrRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request_sized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_GETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_getxattr_in {
			size: 10,
		}))
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(GetxattrRequest, buf);

	assert_eq!(req.size(), Some(num::NonZeroUsize::new(10).unwrap()));
	assert_eq!(req.name(), c"hello.world!");
}

#[test]
fn request_unsized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_GETXATTR;
			h.nodeid = 123;
		})
		.push_sized(&kernel::fuse_getxattr_in::new())
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(GetxattrRequest, buf);

	assert_eq!(req.size(), None);
	assert_eq!(req.name(), c"hello.world!");
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, GetxattrRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_GETXATTR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_getxattr_in {
			size: 11,
		}))
		.push_bytes(b"hello.world!\x00")
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"GetxattrRequest {\n",
			"    node_id: 1,\n",
			"    size: Some(11),\n",
			"    name: \"hello.world!\",\n",
			"}",
		),
	);
}
