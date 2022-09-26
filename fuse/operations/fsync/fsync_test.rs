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

use fuse::operations::fsync::{FsyncRequest, FsyncResponse};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_FSYNC;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_fsync_in {
			fh: 3,
			fsync_flags: 0x1,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(FsyncRequest, buf);

	assert_eq!(req.handle(), 3);
	assert_eq!(req.flags().get(fuse::FsyncRequestFlag::FDATASYNC), true);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, FsyncRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_FSYNC;
			h.nodeid = fuse_kernel::FUSE_ROOT_ID;
		})
		.push_sized(&fuse_kernel::fuse_fsync_in {
			fh: 3,
			fsync_flags: 0x1,
			padding: 0,
		})
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FsyncRequest {\n",
			"    node_id: 1,\n",
			"    handle: 3,\n",
			"    flags: FsyncRequestFlags {\n",
			"        FDATASYNC,\n",
			"    },\n",
			"}",
		),
	);
}

#[test]
fn response_empty() {
	let resp = FsyncResponse::new();
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
	let response = FsyncResponse::new();
	assert_eq!(format!("{:#?}", response), "FsyncResponse",);
}
