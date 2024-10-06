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

use fuse::kernel;
use fuse::operations::access::{AccessRequest, AccessResponse};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_ACCESS;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_access_in {
			mask: 0xFF,
		}))
		.build_aligned();
	let req = decode_request!(AccessRequest, buf);

	assert_eq!(req.mask(), 0xFF);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, AccessRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_ACCESS;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&kernel::fuse_access_in::new())
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"AccessRequest {\n",
			"    node_id: 1,\n",
			"    mask: 0,\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let resp = AccessResponse::new();
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_out_header {
				len: size_of::<kernel::fuse_out_header>() as u32,
				unique: 0xAABBCCDD,
			}))
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let response = AccessResponse::new();
	assert_eq!(format!("{:#?}", response), "AccessResponse",);
}
