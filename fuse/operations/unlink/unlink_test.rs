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

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{UnlinkRequest, UnlinkResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_UNLINK;
			h.nodeid = 100;
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let request = decode_request!(UnlinkRequest, buf);

	let expect: &[u8] = b"hello.world!";
	assert_eq!(request.parent_id(), NodeId::new(100).unwrap());
	assert_eq!(request.name(), expect);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_UNLINK;
			h.nodeid = 100;
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();
	let request = decode_request!(UnlinkRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"UnlinkRequest {\n",
			"    parent_id: 100,\n",
			"    name: \"hello.world!\",\n",
			"}",
		),
	);
}

#[test]
fn response_empty() {
	let resp = UnlinkResponse::new();
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

#[test]
fn response_impl_debug() {
	let response = UnlinkResponse::new();

	assert_eq!(format!("{:#?}", response), "UnlinkResponse");
}
