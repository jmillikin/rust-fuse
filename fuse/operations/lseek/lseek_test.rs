// Copyright 2021 John Millikin and the rust-fuse contributors.
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

use super::{LseekRequest, LseekResponse, LseekWhence};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_LSEEK;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_lseek_in {
			fh: 12,
			offset: 34,
			whence: LseekWhence::SEEK_DATA.0,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(LseekRequest, buf);

	assert_eq!(req.handle(), 12);
	assert_eq!(req.offset(), 34);
	assert_eq!(req.whence(), LseekWhence::SEEK_DATA);
}

#[test]
fn request_impl_debug() {
	let request = LseekRequest {
		raw: &fuse_kernel::fuse_lseek_in {
			fh: 12,
			offset: 34,
			whence: 3,
			padding: 0,
		},
		node_id: crate::ROOT_ID,
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"LseekRequest {\n",
			"    node_id: 1,\n",
			"    handle: 12,\n",
			"    offset: 34,\n",
			"    whence: SEEK_DATA,\n",
			"}",
		),
	);
}

#[test]
fn response_empty() {
	let mut resp = LseekResponse::new();
	resp.set_offset(4096);
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_lseek_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_lseek_out { offset: 4096 })
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = LseekResponse::new();
	response.set_offset(4096);
	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"LseekResponse {\n",
			"    offset: 4096,\n",
			"}",
		),
	);
}
