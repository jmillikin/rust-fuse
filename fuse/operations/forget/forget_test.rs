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

use fuse::node;
use fuse::operations::forget::{ForgetRequest, ForgetRequestItem};

use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request_single() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_FORGET;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_forget_in { nlookup: 456 })
		.build_aligned();

	let req = decode_request!(ForgetRequest, buf);

	let items: Vec<ForgetRequestItem> = req.items().collect();
	assert_eq!(items.len(), 1);
	assert_eq!(items[0].node_id(), node::Id::new(123).unwrap());
	assert_eq!(items[0].lookup_count(), 456);
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

	let req = decode_request!(ForgetRequest, buf);

	let items: Vec<ForgetRequestItem> = req.items().collect();
	assert_eq!(items.len(), 2);
	assert_eq!(items[0].node_id(), node::Id::new(12).unwrap());
	assert_eq!(items[0].lookup_count(), 34);
	assert_eq!(items[1].node_id(), node::Id::new(56).unwrap());
	assert_eq!(items[1].lookup_count(), 78);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_FORGET;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_forget_in { nlookup: 456 })
		.build_aligned();

	let request = decode_request!(ForgetRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ForgetRequest {\n",
			"    items: [\n",
			"        ForgetRequestItem {\n",
			"            node_id: 123,\n",
			"            lookup_count: 456,\n",
			"        },\n",
			"    ],\n",
			"}",
		),
	);
}
