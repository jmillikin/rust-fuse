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

use core::marker::PhantomData;
use core::mem::size_of;

use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{ReadlinkRequest, ReadlinkResponse};

#[test]
fn request_empty() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READLINK;
			h.nodeid = 123;
		})
		.build_aligned();

	let _req = decode_request!(ReadlinkRequest, buf);
}

#[test]
fn request_impl_debug() {
	let request = &ReadlinkRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ReadlinkRequest {\n",
			"    node_id: 1,\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let name = NodeName::from_bytes(b"hello.world!").unwrap();
	let resp = ReadlinkResponse::from_name(name);
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>() + 12) as u32,
				error: 0,
				unique: 0,
			})
			.push_bytes(b"hello.world!")
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let name = NodeName::from_bytes(b"hello.world!").unwrap();
	let response = ReadlinkResponse::from_name(name);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ReadlinkResponse {\n",
			r#"    target: "hello.world!","#, "\n",
			"}",
		),
	);
}
