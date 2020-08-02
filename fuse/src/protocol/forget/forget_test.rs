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

use crate::internal::testutil::MessageBuilder;
use crate::protocol::node;
use crate::protocol::prelude::*;

use super::ForgetRequest;

#[test]
fn request_single() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_FORGET;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_forget_in { nlookup: 456 })
		.build_aligned();

	let req: ForgetRequest = decode_request!(buf);

	let nodes = req.nodes();
	assert_eq!(nodes.len(), 1);
	assert_eq!(nodes[0].id(), node::NodeId::new(123));
	assert_eq!(nodes[0].count(), 456);
}

#[test]
fn request_batch() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_BATCH_FORGET)
		.push_sized(&fuse_kernel::fuse_batch_forget_in { count: 2, dummy: 0 })
		.push_sized(&fuse_kernel::fuse_forget_one {
			nodeid: 12,
			nlookup: 34,
		})
		.push_sized(&fuse_kernel::fuse_forget_one {
			nodeid: 56,
			nlookup: 78,
		})
		.build_aligned();

	let req: ForgetRequest = decode_request!(buf);

	let nodes = req.nodes();
	assert_eq!(nodes.len(), 2);
	assert_eq!(nodes[0].id(), node::NodeId::new(12));
	assert_eq!(nodes[0].count(), 34);
	assert_eq!(nodes[1].id(), node::NodeId::new(56));
	assert_eq!(nodes[1].count(), 78);
}
