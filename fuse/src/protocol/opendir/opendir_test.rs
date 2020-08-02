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

use super::{OpendirRequest, OpendirResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_OPENDIR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_open_in {
			flags: 0xFF,
			unused: 0,
		})
		.build_aligned();

	let req: OpendirRequest = decode_request!(buf);

	assert_eq!(req.flags(), 0xFF);
}

#[test]
fn response() {
	let mut resp = OpendirResponse::new();
	resp.set_handle(123);
	resp.set_flags(0xFE);

	let encoded = encode_response!(resp, {
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
				open_flags: 0xFE,
				padding: 0,
			})
			.build()
	);
}
