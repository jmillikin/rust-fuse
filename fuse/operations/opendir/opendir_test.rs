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
use fuse::server::OpendirRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_OPENDIR;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_open_in {
			flags: 0xFF,
			open_flags: 0, // TODO
		}))
		.build_aligned();

	let req = decode_request!(OpendirRequest, buf);

	assert_eq!(req.open_flags(), 0xFF);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, OpendirRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_OPENDIR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_open_in {
			flags: 0x1,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"OpendirRequest {\n",
			"    node_id: 1,\n",
			"    flags: OpendirRequestFlags {},\n",
			"    open_flags: 0x00000001,\n",
			"}",
		),
	);
}
