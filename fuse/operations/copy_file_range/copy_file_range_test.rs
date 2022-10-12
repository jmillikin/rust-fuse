// Copyright 2022 John Millikin and the rust-fuse contributors.
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

use fuse::node;
use fuse::operations::copy_file_range::{
	CopyFileRangeRequest,
	CopyFileRangeRequestFlags,
	CopyFileRangeResponse,
};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_COPY_FILE_RANGE;
			h.nodeid = 10;
		})
		.push_sized(&fuse_kernel::fuse_copy_file_range_in {
			fh_in: 11,
			off_in: 12,
			nodeid_out: 13,
			fh_out: 14,
			off_out: 15,
			len: 16,
			flags: 0,
		})
		.build_aligned();

	let req = decode_request!(CopyFileRangeRequest, buf);

	assert_eq!(req.input_node_id(), node::Id::new(10).unwrap());
	assert_eq!(req.input_handle(), 11);
	assert_eq!(req.input_offset(), 12);
	assert_eq!(req.output_node_id(), node::Id::new(13).unwrap());
	assert_eq!(req.output_handle(), 14);
	assert_eq!(req.output_offset(), 15);
	assert_eq!(req.len(), 16);
	assert_eq!(req.flags(), CopyFileRangeRequestFlags::new());
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, CopyFileRangeRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_COPY_FILE_RANGE;
			h.nodeid = 10;
		})
		.push_sized(&fuse_kernel::fuse_copy_file_range_in {
			fh_in: 11,
			off_in: 12,
			nodeid_out: 13,
			fh_out: 14,
			off_out: 15,
			len: 16,
			flags: 0,
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"CopyFileRangeRequest {\n",
			"    input_node_id: 10,\n",
			"    input_handle: 11,\n",
			"    input_offset: 12,\n",
			"    output_node_id: 13,\n",
			"    output_handle: 14,\n",
			"    output_offset: 15,\n",
			"    len: 16,\n",
			"    flags: CopyFileRangeRequestFlags {},\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let mut resp = CopyFileRangeResponse::new();
	resp.set_size(123);

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_write_out>()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_write_out {
				size: 123,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = CopyFileRangeResponse::new();
	response.set_size(123);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"CopyFileRangeResponse {\n",
			"    size: 123,\n",
			"}",
		),
	);
}
