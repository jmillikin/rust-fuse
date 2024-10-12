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
use fuse::server::MkdirRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

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

	assert_eq!(req.parent_id(), fuse::NodeId::new(100).unwrap());
	assert_eq!(req.name(), "hello.world!");
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
