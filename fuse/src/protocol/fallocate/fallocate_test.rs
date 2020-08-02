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

use super::{FallocateRequest, FallocateResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_FALLOCATE)
		.push_sized(&fuse_kernel::fuse_fallocate_in {
			fh: 12,
			offset: 34,
			length: 56,
			mode: 78,
			padding: 0,
		})
		.build_aligned();

	let req: FallocateRequest = decode_request!(buf);

	assert_eq!(req.handle(), 12);
	assert_eq!(req.offset(), 34);
	assert_eq!(req.length(), 56);
	assert_eq!(req.mode(), 78);
}

#[test]
fn response_empty() {
	let resp = FallocateResponse::new();
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
