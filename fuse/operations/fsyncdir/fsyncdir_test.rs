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
use fuse::server::FsyncdirRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_FSYNCDIR;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_fsync_in {
			fh: 3,
			fsync_flags: 0x1,
		}))
		.build_aligned();

	let req = decode_request!(FsyncdirRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 3);
	assert_eq!(req.flags().get(fuse::FsyncdirRequestFlag::FDATASYNC), true);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, FsyncdirRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_FSYNCDIR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_fsync_in {
			fh: 3,
			fsync_flags: 0x1,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FsyncdirRequest {\n",
			"    node_id: 1,\n",
			"    handle: 3,\n",
			"    flags: FsyncdirRequestFlags {\n",
			"        FDATASYNC,\n",
			"    },\n",
			"}",
		),
	);
}
