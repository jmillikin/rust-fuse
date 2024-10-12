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
use fuse::server::RenameRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request_rename() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RENAME;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_rename_in {
			newdir: 456,
		}))
		.push_bytes(b"old\x00")
		.push_bytes(b"new\x00")
		.build_aligned();

	let request = decode_request!(RenameRequest, buf);

	assert_eq!(request.old_name(), "old");
	assert_eq!(request.new_name(), "new");
	assert_eq!(request.old_directory_id(), fuse::NodeId::new(123).unwrap());
	assert_eq!(request.new_directory_id(), fuse::NodeId::new(456).unwrap());
	assert_eq!(request.rename_flags(), 0);
}

#[test]
fn request_rename2() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RENAME2;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_rename2_in {
			newdir: 456,
			flags: 0b111,
		}))
		.push_bytes(b"old\x00")
		.push_bytes(b"new\x00")
		.build_aligned();

	let request = decode_request!(RenameRequest, buf);

	assert_eq!(request.old_name(), "old");
	assert_eq!(request.new_name(), "new");
	assert_eq!(request.old_directory_id(), fuse::NodeId::new(123).unwrap());
	assert_eq!(request.new_directory_id(), fuse::NodeId::new(456).unwrap());
	assert_eq!(request.rename_flags(), 0b111);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_RENAME2;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_rename2_in {
			newdir: 456,
			flags: 0b111,
		}))
		.push_bytes(b"old\x00")
		.push_bytes(b"new\x00")
		.build_aligned();
	let request = decode_request!(RenameRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"RenameRequest {\n",
			"    old_directory_id: 123,\n",
			"    old_name: \"old\",\n",
			"    new_directory_id: 456,\n",
			"    new_name: \"new\",\n",
			"    rename_flags: 0x00000007,\n",
			"}",
		),
	);
}
