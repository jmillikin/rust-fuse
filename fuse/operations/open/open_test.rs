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
use fuse::server::OpenRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_OPEN;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_open_in {
			flags: 0xFF,
			open_flags: kernel::FUSE_OPEN_KILL_SUIDGID,
		}))
		.build_aligned();

	let req = decode_request!(OpenRequest, buf);

	assert_eq!(req.flags().get(fuse::OpenRequestFlag::KILL_SUIDGID), true);
	assert_eq!(req.open_flags(), 0xFF);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, OpenRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_OPEN;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_open_in {
			flags: 0xFF,
			open_flags: kernel::FUSE_OPEN_KILL_SUIDGID,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"OpenRequest {\n",
			"    node_id: 1,\n",
			"    flags: OpenRequestFlags {\n",
			"        KILL_SUIDGID,\n",
			"    },\n",
			"    open_flags: 0x000000FF,\n",
			"}",
		),
	);
}
