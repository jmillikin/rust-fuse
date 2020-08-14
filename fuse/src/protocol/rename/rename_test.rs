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
use crate::protocol::prelude::*;

use super::{RenameRequest, RenameResponse};

#[test]
fn request_rename() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_RENAME;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_rename_in { newdir: 456 })
		.push_bytes(b"old\x00")
		.push_bytes(b"new\x00")
		.build_aligned();

	let req: RenameRequest = decode_request!(buf);

	let expect_old = CString::new("old").unwrap();
	let expect_new = CString::new("new").unwrap();
	assert_eq!(req.old_name(), expect_old.as_ref());
	assert_eq!(req.new_name(), expect_new.as_ref());
	assert_eq!(req.old_dir(), NodeId::new(123).unwrap());
	assert_eq!(req.new_dir(), NodeId::new(456).unwrap());
	assert_eq!(req.exchange(), false);
	assert_eq!(req.no_replace(), false);
}

#[test]
fn request_rename2() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_RENAME2;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_rename2_in {
			newdir: 456,
			flags: 0xFF,
			padding: 0,
		})
		.push_bytes(b"old\x00")
		.push_bytes(b"new\x00")
		.build_aligned();

	let req: RenameRequest = decode_request!(buf);

	let expect_old = CString::new("old").unwrap();
	let expect_new = CString::new("new").unwrap();
	assert_eq!(req.old_name(), expect_old.as_ref());
	assert_eq!(req.new_name(), expect_new.as_ref());
	assert_eq!(req.old_dir(), NodeId::new(123).unwrap());
	assert_eq!(req.new_dir(), NodeId::new(456).unwrap());
	assert_eq!(req.exchange(), true);
	assert_eq!(req.no_replace(), true);
}

#[test]
fn response_empty() {
	let resp = RenameResponse::new();
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: size_of::<fuse_kernel::fuse_out_header>() as u32,
				error: 0,
				unique: 0,
			})
			.build()
	);
}
