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
use crate::protocol::prelude::*;

use super::{RmdirRequest, RmdirResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_RMDIR)
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req: RmdirRequest = decode_request!(buf);

	let expect = CString::new("hello.world!").unwrap();
	assert_eq!(req.name(), expect.as_ref());
}

#[test]
fn response_empty() {
	let resp = RmdirResponse::new();
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
