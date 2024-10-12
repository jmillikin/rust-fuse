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
use fuse::server::SymlinkRequest;

use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SYMLINK;
			h.nodeid = 100;
		})
		.push_bytes(b"link content\x00")
		.push_bytes(b"link name\x00")
		.build_aligned();
	let request = decode_request!(SymlinkRequest, buf);

	assert_eq!(request.parent_id(), fuse::NodeId::new(100).unwrap());
	assert_eq!(request.name(), "link name");
	assert_eq!(request.content(), c"link content");
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_SYMLINK;
			h.nodeid = 100;
		})
		.push_bytes(b"link content\x00")
		.push_bytes(b"link name\x00")
		.build_aligned();
	let request = decode_request!(SymlinkRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"SymlinkRequest {\n",
			"    parent_id: 100,\n",
			"    name: \"link name\",\n",
			"    content: \"link content\",\n",
			"}",
		),
	);
}
