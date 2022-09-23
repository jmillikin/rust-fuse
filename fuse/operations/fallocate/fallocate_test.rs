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

use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;

use super::{FallocateRequest, FallocateResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_FALLOCATE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_fallocate_in {
			fh: 12,
			offset: 34,
			length: 56,
			mode: 0b11,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(FallocateRequest, buf);

	assert_eq!(req.handle(), 12);
	assert_eq!(req.offset(), 34);
	assert_eq!(req.length(), 56);
	assert_eq!(req.mode().keep_size, true);
	assert_eq!(req.mode().punch_hole, true);
}

#[test]
fn request_impl_debug() {
	let request = FallocateRequest {
		raw: &fuse_kernel::fuse_fallocate_in {
			fh: 123,
			offset: 1024,
			length: 4096,
			mode: 0b11,
			padding: 0,
		},
		node_id: crate::ROOT_ID,
		mode: super::FallocateMode::from_bits(0b11),
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FallocateRequest {\n",
			"    node_id: 1,\n",
			"    handle: 123,\n",
			"    offset: 1024,\n",
			"    length: 4096,\n",
			"    mode: FallocateMode {\n",
			"        keep_size: true,\n",
			"        punch_hole: true,\n",
			"        collapse_range: false,\n",
			"        zero_range: false,\n",
			"        insert_range: false,\n",
			"        unshare_range: false,\n",
			"    },\n",
			"}",
		),
	);
}

#[test]
fn response_empty() {
	let resp = FallocateResponse::new();
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
	let response = FallocateResponse::new();
	assert_eq!(format!("{:#?}", response), "FallocateResponse");
}
