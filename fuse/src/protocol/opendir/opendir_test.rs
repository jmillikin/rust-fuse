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

use super::{OpendirRequest, OpendirResponse, OpendirResponseFlags};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_OPENDIR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_open_in {
			flags: 0xFF,
			open_flags: 0, // TODO
		})
		.build_aligned();

	let req: OpendirRequest = decode_request!(buf);

	assert_eq!(req.flags(), 0xFF);
}

#[test]
fn request_impl_debug() {
	let request = &OpendirRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
		flags: 0x1,
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"OpendirRequest {\n",
			"    node_id: 1,\n",
			"    flags: 0x00000001,\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let mut response = OpendirResponse::new();
	response.set_handle(123);
	response.flags_mut().keep_cache = true;

	let encoded = encode_response!(response, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_open_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_open_out {
				fh: 123,
				open_flags: 0x2,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = OpendirResponse::new();
	response.set_handle(123);
	response.flags_mut().keep_cache = true;

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"OpendirResponse {\n",
			"    handle: 123,\n",
			"    flags: OpendirResponseFlags {\n",
			"        keep_cache: true,\n",
			"        nonseekable: false,\n",
			"    },\n",
			"}",
		),
	);
}

#[test]
fn open_flags() {
	// Flag sets render as a struct, with unknown flags falling back
	// to hex.
	assert_eq!(
		format!("{:#?}", OpendirResponseFlags::from_bits(0x2 | (1u32 << 31))),
		concat!(
			"OpendirResponseFlags {\n",
			"    keep_cache: true,\n",
			"    nonseekable: false,\n",
			"    0x80000000: true,\n",
			"}",
		),
	);
}
