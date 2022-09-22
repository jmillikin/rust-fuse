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

use super::{SetattrRequest, SetattrResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_SETATTR)
		.push_sized(&fuse_kernel::fuse_setattr_in {
			valid: 0xFFFF,
			padding: 0,
			fh: 1,
			size: 2,
			lock_owner: 3,
			atime: 4,
			mtime: 5,
			ctime: 6,
			atimensec: 7,
			mtimensec: 8,
			ctimensec: 9,
			mode: u32::from(FileType::Regular | 0o644),
			unused4: 11,
			uid: 12,
			gid: 13,
			unused5: 14,
		})
		.build_aligned();

	let req = decode_request!(SetattrRequest, buf);

	assert_eq!(req.handle(), Some(1));
	assert_eq!(req.size(), Some(2));
	assert_eq!(req.lock_owner(), Some(3));
	assert_eq!(req.atime(), Some(super::systime(4, 7)));
	assert_eq!(req.atime_now(), true);
	assert_eq!(req.mtime(), Some(super::systime(5, 8)));
	assert_eq!(req.mtime_now(), true);
	assert_eq!(req.ctime(), Some(super::systime(6, 9)));
	assert_eq!(req.mode(), Some(FileType::Regular | 0o644));
	assert_eq!(req.user_id(), Some(12));
	assert_eq!(req.group_id(), Some(13));
}

#[test]
fn response_v7p1() {
	return; // SKIP TEST

	let resp: SetattrResponse = todo!();
	/*
		let resp = SetattrRequest {
			header: &HEADER,
			raw: &Default::default(),
		}.new_response();
	*/

	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 0,
				attr_valid_nsec: 0,
				dummy: 0,
				attr: fuse_kernel::fuse_attr::zeroed(),
			})
			.unpush(
				size_of::<fuse_kernel::fuse_attr_out>()
					- fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE
			)
			.build()
	);
}

#[test]
fn response_v7p9() {
	return; // SKIP TEST

	let resp: SetattrResponse = todo!();
	/*
	let mut resp = SetattrRequest {
		header: &HEADER,
		raw: &Default::default(),
		}.new_response();
		*/

	resp.attr_mut().set_size(999);
	resp.set_cache_duration(time::Duration::new(123, 456));

	let encoded = encode_response!(resp, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_attr_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_attr_out {
				attr_valid: 123,
				attr_valid_nsec: 456,
				dummy: 0,
				attr: fuse_kernel::fuse_attr {
					size: 999,
					..fuse_kernel::fuse_attr::zeroed()
				},
			})
			.build()
	);
}
