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

use fuse::kernel;
use fuse::server::FallocateRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_FALLOCATE;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_fallocate_in {
			fh: 12,
			offset: 34,
			length: 56,
			mode: 0b11,
		}))
		.build_aligned();

	let req = decode_request!(FallocateRequest, buf);

	assert_eq!(req.handle(), 12);
	assert_eq!(req.offset(), 34);
	assert_eq!(req.length(), 56);
	assert_eq!(req.fallocate_flags(), 0b11);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, FallocateRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_FALLOCATE;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_fallocate_in {
			fh: 123,
			offset: 1024,
			length: 4096,
			mode: 0b11,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FallocateRequest {\n",
			"    node_id: 1,\n",
			"    handle: 123,\n",
			"    offset: 1024,\n",
			"    length: 4096,\n",
			"    fallocate_flags: 0x00000003,\n",
			"}",
		),
	);
}
