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

use core::marker::PhantomData;
use core::mem;
use core::mem::size_of;
use core::num;

use crate::FileType;
use crate::NodeId;
use crate::NodeName;
use crate::internal::fuse_kernel;
use crate::internal::testutil::MessageBuilder;
use crate::operations::read::fuse_read_in_v7p1;

use super::{ReaddirRequest, ReaddirResponse};

#[test]
fn readdir_request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READDIR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_read_in_v7p1 {
			fh: 123,
			offset: 45,
			size: 4096,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(ReaddirRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.cursor(), num::NonZeroU64::new(45));
	assert_eq!(req.size(), 4096);
}

#[test]
fn readdir_request_v7p9() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READDIR;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 4096,
			read_flags: 0,
			lock_owner: 0,
			flags: 67,
			padding: 0,
		})
		.build_aligned();

	let req = decode_request!(ReaddirRequest, buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.cursor(), num::NonZeroU64::new(45));
	assert_eq!(req.opendir_flags(), 67);
	assert_eq!(req.size(), 4096);
}

#[test]
fn request_impl_debug() {
	let request = &ReaddirRequest {
		phantom: PhantomData,
		node_id: crate::ROOT_ID,
		size: 1,
		cursor: num::NonZeroU64::new(2),
		handle: 3,
		opendir_flags: 0x4,
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ReaddirRequest {\n",
			"    node_id: 1,\n",
			"    size: 1,\n",
			"    cursor: Some(2),\n",
			"    handle: 3,\n",
			"    opendir_flags: 0x00000004,\n",
			"}",
		),
	);
}

#[test]
fn readdir_response_heap() {
	let max_size = size_of::<fuse_kernel::fuse_dirent>() + 12;
	let mut resp = ReaddirResponse::with_max_size(max_size as u32);
	readdir_response_test_impl(&mut resp);
}

#[test]
fn readdir_response_stack() {
	let mut buf = [0u8; 1024];
	let aligned = match buf.as_ptr().align_offset(mem::align_of::<u64>()) {
		0 => &mut buf,
		offset => {
			let (_, aligned) = buf.split_at_mut(offset);
			aligned
		},
	};

	let max_size = size_of::<fuse_kernel::fuse_dirent>() + 12;
	let (sized_buf, _) = aligned.split_at_mut(max_size);
	let mut resp = ReaddirResponse::with_capacity(sized_buf);
	readdir_response_test_impl(&mut resp);
}

fn readdir_response_test_impl(resp: &mut ReaddirResponse) {
	// Adding a dirent fails if there's not enough capacity.
	{
		let node_id = NodeId::new(100).unwrap();
		let name = NodeName::from_bytes(b"123456789ABCDEF").unwrap();
		let cursor = num::NonZeroU64::new(1).unwrap();
		let opt_dirent = resp.try_add_entry(node_id, name, cursor);
		assert!(opt_dirent.is_err());
	}

	// Dirent capacity takes 8-byte name padding into account.
	{
		let node_id = NodeId::new(100).unwrap();
		let name = NodeName::from_bytes(b"123456789").unwrap();
		let cursor = num::NonZeroU64::new(1).unwrap();
		let opt_dirent = resp.try_add_entry(node_id, name, cursor);
		assert!(opt_dirent.is_err());
	}

	// Adding a dirent works if there's enough capacity.
	{
		let node_id = NodeId::new(100).unwrap();
		let name = NodeName::from_bytes(b"foobar").unwrap();
		let cursor = num::NonZeroU64::new(1).unwrap();
		let dirent = resp.try_add_entry(node_id, name, cursor).unwrap();

		assert_eq!(dirent.cursor(), cursor);
		assert_eq!(dirent.file_type(), FileType::Unknown);

		dirent.set_file_type(FileType::Regular);
	}

	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_dirent>()
					+ 8) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_dirent {
				ino: 100,
				off: 1,
				namelen: 6,
				r#type: 8,
				name: [0u8; 0],
			})
			.push_bytes(b"foobar\0\0")
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = ReaddirResponse::with_max_size(1024);

	{
		let node_id = NodeId::new(100).unwrap();
		let name = NodeName::from_bytes(b"hello.txt").unwrap();
		let cursor = num::NonZeroU64::new(1).unwrap();
		response
			.add_entry(node_id, name, cursor)
			.set_file_type(FileType::Regular);
	}

	{
		let node_id = NodeId::new(101).unwrap();
		let name = NodeName::from_bytes(b"world.txt").unwrap();
		let cursor = num::NonZeroU64::new(2).unwrap();
		response
			.add_entry(node_id, name, cursor)
			.set_file_type(FileType::Regular);
	}

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"ReaddirResponse {\n",
			"    entries: [\n",
			"        ReaddirEntry {\n",
			"            node_id: 100,\n",
			"            cursor: 1,\n",
			"            file_type: Regular,\n",
			"            name: \"hello.txt\",\n",
			"        },\n",
			"        ReaddirEntry {\n",
			"            node_id: 101,\n",
			"            cursor: 2,\n",
			"            file_type: Regular,\n",
			"            name: \"world.txt\",\n",
			"        },\n",
			"    ],\n",
			"}",
		),
	);
}
