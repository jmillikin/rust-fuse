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

use core::mem::size_of;
use core::num;

use fuse::kernel;
use fuse::server::{
	ReaddirEntriesWriter,
	ReaddirEntry,
	ReaddirRequest,
};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn readdir_request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_READDIR;
			h.nodeid = 123;
		})
		.push_sized(&123u64) // fuse_read_in::fh
		.push_sized(&45u64) // fuse_read_in::offset
		.push_sized(&4096u32) // fuse_read_in::size
		.push_sized(&0u32) // fuse_read_in::padding
		.build_aligned();

	let req = decode_request!(ReaddirRequest, buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), num::NonZeroU64::new(45));
	assert_eq!(req.size(), 4096);
}

#[test]
fn readdir_request_v7p9() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_READDIR;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 4096,
			flags: 67,
		}))
		.build_aligned();

	let req = decode_request!(ReaddirRequest, buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), num::NonZeroU64::new(45));
	assert_eq!(req.open_flags(), 67);
	assert_eq!(req.size(), 4096);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, ReaddirRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_READDIR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_read_in {
			fh: 3,
			offset: 2,
			size: 1,
			flags: 0x4,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ReaddirRequest {\n",
			"    node_id: 1,\n",
			"    size: 1,\n",
			"    offset: Some(2),\n",
			"    handle: 3,\n",
			"    open_flags: 0x00000004,\n",
			"}",
		),
	);
}

#[test]
fn readdir_entries() {
	let max_size = size_of::<kernel::fuse_dirent>() + 12;
	let mut buf = vec![0u8; max_size];
	let mut writer = ReaddirEntriesWriter::new(&mut buf);

	// Adding a dirent fails if there's not enough capacity.
	{
		let node_id = fuse::NodeId::new(100).unwrap();
		let name = fuse::NodeName::new("123456789ABCDEF").unwrap();
		let offset = num::NonZeroU64::new(1).unwrap();
		let entry = ReaddirEntry::new(node_id, name, offset);
		assert!(writer.try_push(&entry).is_err());
	}

	// Dirent capacity takes 8-byte name padding into account.
	{
		let node_id = fuse::NodeId::new(100).unwrap();
		let name = fuse::NodeName::new("123456789").unwrap();
		let offset = num::NonZeroU64::new(1).unwrap();
		let entry = ReaddirEntry::new(node_id, name, offset);
		assert!(writer.try_push(&entry).is_err());
	}

	// Adding a dirent works if there's enough capacity.
	{
		let node_id = fuse::NodeId::new(100).unwrap();
		let name = fuse::NodeName::new("foobar").unwrap();
		let offset = num::NonZeroU64::new(1).unwrap();

		let mut entry = ReaddirEntry::new(node_id, name, offset);
		entry.set_file_type(fuse::FileType::Regular);
		assert!(writer.try_push(&entry).is_ok());
	}

	let entries = writer.into_entries();

	assert_eq!(
		entries.as_bytes(),
		MessageBuilder::new()
			.push_sized(&testutil::new!(kernel::fuse_dirent {
				ino: 100,
				off: 1,
				namelen: 6,
				r#type: 8,
			}))
			.push_bytes(b"foobar\0\0")
			.build()
	);
}

#[test]
fn readdir_entries_debug() {
	let mut buf = vec![0u8; 1024];
	let mut writer = ReaddirEntriesWriter::new(&mut buf);

	{
		let node_id = fuse::NodeId::new(100).unwrap();
		let name = fuse::NodeName::new("hello.txt").unwrap();
		let offset = num::NonZeroU64::new(1).unwrap();

		let mut entry = ReaddirEntry::new(node_id, name, offset);
		entry.set_file_type(fuse::FileType::Regular);
		assert!(writer.try_push(&entry).is_ok());
	}

	{
		let node_id = fuse::NodeId::new(101).unwrap();
		let name = fuse::NodeName::new("world.txt").unwrap();
		let offset = num::NonZeroU64::new(2).unwrap();

		let mut entry = ReaddirEntry::new(node_id, name, offset);
		entry.set_file_type(fuse::FileType::Regular);
		assert!(writer.try_push(&entry).is_ok());
	}

	assert_eq!(
		format!("{:#?}", writer.into_entries()),
		concat!(
			"[\n",
			"    ReaddirEntry {\n",
			"        node_id: 100,\n",
			"        offset: 1,\n",
			"        file_type: Some(Regular),\n",
			"        name: \"hello.txt\",\n",
			"    },\n",
			"    ReaddirEntry {\n",
			"        node_id: 101,\n",
			"        offset: 2,\n",
			"        file_type: Some(Regular),\n",
			"        name: \"world.txt\",\n",
			"    },\n",
			"]",
		),
	);
}
