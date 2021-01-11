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

use super::{AccessRequest, AccessResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_ACCESS;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_access_in {
			mask: 0xFF,
			padding: 0,
		})
		.build_aligned();
	let req: AccessRequest = decode_request!(buf);

	assert_eq!(req.mask(), 0xFF);
}

#[test]
fn request_impl_debug() {
	let request = &AccessRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
		mask: 0,
	};

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
	let response = AccessResponse::new();
	assert_eq!(format!("{:#?}", response), "AccessResponse",);
}
