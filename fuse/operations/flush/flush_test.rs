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
use fuse::server::FlushRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_FLUSH;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_flush_in {
			fh: 123,
			lock_owner: 456,
		}))
		.build_aligned();

	let req = decode_request!(FlushRequest, buf);

	assert_eq!(req.handle(), 123);
	assert_eq!(req.lock_owner(), fuse::LockOwner(456));
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, FlushRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_FLUSH;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_flush_in {
			fh: 12,
			lock_owner: 34,
		}))
	});
	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FlushRequest {\n",
			"    node_id: 1,\n",
			"    handle: 12,\n",
			"    lock_owner: 0x0000000000000022,\n",
			"}",
		),
	);
}
