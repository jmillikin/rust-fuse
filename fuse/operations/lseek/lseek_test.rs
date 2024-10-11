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

use fuse::kernel;
use fuse::server::LseekRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LSEEK;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_lseek_in {
			fh: 12,
			offset: 34,
			whence: 3,
		}))
		.build_aligned();

	let req = decode_request!(LseekRequest, buf);

	assert_eq!(req.handle(), 12);
	assert_eq!(req.offset(), 34);
	assert_eq!(req.whence(), fuse::LseekWhence::SEEK_DATA);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, LseekRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LSEEK;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_lseek_in {
			fh: 12,
			offset: 34,
			whence: 3,
		}))
	});

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
