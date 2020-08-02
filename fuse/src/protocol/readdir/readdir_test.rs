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
use crate::protocol::read::fuse_read_in_v7p1;

use super::{Dirent, ReaddirRequest, ReaddirResponse};

const DUMMY_READ_FLAG: u32 = 0x80000000;

#[test]
fn readdir_request_v7p1() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_READDIR)
		.push_sized(&fuse_read_in_v7p1 {
			fh: 123,
			offset: 45,
			size: 4096,
			padding: 0,
		})
		.build_aligned();

	let req: ReaddirRequest = decode_request!(buf, {
		protocol_version: (7, 1),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.lock_owner(), None);
	assert_eq!(req.size(), 4096);
}

#[test]
fn readdir_request_v7p9() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_READDIR)
		.push_sized(&fuse_kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 4096,
			read_flags: DUMMY_READ_FLAG | fuse_kernel::FUSE_READ_LOCKOWNER,
			lock_owner: 1234,
			flags: 67,
			padding: 0,
		})
		.build_aligned();

	let req: ReaddirRequest = decode_request!(buf, {
		protocol_version: (7, 9),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.flags(), 67);
	assert_eq!(req.lock_owner(), Some(1234));
	assert_eq!(req.size(), 4096);
}

#[test]
fn readdirplus_request() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_READDIRPLUS)
		.push_sized(&fuse_kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 4096,
			read_flags: DUMMY_READ_FLAG | fuse_kernel::FUSE_READ_LOCKOWNER,
			lock_owner: 1234,
			flags: 67,
			padding: 0,
		})
		.build_aligned();

	let req: ReaddirRequest = decode_request!(buf, {
		protocol_version: (7, 21),
	});

	assert_eq!(req.handle(), 123);
	assert_eq!(req.offset(), 45);
	assert_eq!(req.flags(), 67);
	assert_eq!(req.lock_owner(), Some(1234));
	assert_eq!(req.size(), 4096);
}

#[test]
fn readdir_response() {
	let mut resp: ReaddirResponse = todo!();
	/*
	let mut resp = ReaddirRequest {
		header: &READDIR_HEADER,
		raw: fuse_kernel::fuse_read_in {
			fh: 0,
			offset: 0,
			size: 24 /* size_of<fuse_dirent> */ + 12,
			read_flags: 0,
			lock_owner: 0,
			flags: 0,
			padding: 0,
		},
		}.new_response();
		*/

	assert_eq!(resp.opcode, fuse_kernel::FUSE_READDIR);
	assert_eq!(resp.response_size, 0);
	assert_eq!(resp.max_response_size, 36);

	// Adding a dirent fails if there's not enough capacity.
	{
		let node_id = node::NodeId::new(100).unwrap();
		let name = CString::new("123456789ABCDEF").unwrap();
		let opt_dirent = resp.push(node_id, 1 /* offset */, &name);
		assert!(opt_dirent.is_none());
	}

	// Dirent capacity takes 8-byte name padding into account.
	{
		let node_id = node::NodeId::new(100).unwrap();
		let name = CString::new("123456789").unwrap();
		let opt_dirent = resp.push(node_id, 1 /* offset */, &name);
		assert!(opt_dirent.is_none());
	}

	// Adding a dirent works if there's enough capacity.
	{
		let node_id = node::NodeId::new(100).unwrap();
		let name = CString::new("foobar").unwrap();
		let dirent = resp.push(node_id, 1 /* offset */, &name).unwrap();

		assert_eq!(dirent.offset(), 1);
		assert_eq!(dirent.node_kind(), node::NodeKind::UNKNOWN);

		let mut dirent = dirent;
		assert!(dirent.node_mut().is_none());
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
				r#type: 0,
				name: [0u8; 0],
			})
			.push_bytes(b"foobar\0\0")
			.build()
	);
}

#[test]
fn readdirplus_response() {
	let mut resp: ReaddirResponse = todo!();
	/*
	let mut resp = ReaddirRequest {
		header: &READDIRPLUS_HEADER,
		raw: fuse_kernel::fuse_read_in {
			fh: 0,
			offset: 0,
			size: 152 /* size_of<fuse_direntplus> */ + 12,
			read_flags: 0,
			lock_owner: 0,
			flags: 0,
			padding: 0,
		},
		}.new_response();
		*/

	assert_eq!(resp.opcode, fuse_kernel::FUSE_READDIRPLUS);
	assert_eq!(resp.response_size, 0);
	assert_eq!(resp.max_response_size, 164);

	// Adding a dirent fails if there's not enough capacity.
	{
		let node_id = node::NodeId::new(100).unwrap();
		let name = CString::new("123456789ABCDEF").unwrap();
		let opt_dirent = resp.push(node_id, 1 /* offset */, &name);
		assert!(opt_dirent.is_none());
	}

	// Dirent capacity takes 8-byte name padding into account.
	{
		let node_id = node::NodeId::new(100).unwrap();
		let name = CString::new("123456789").unwrap();
		let opt_dirent = resp.push(node_id, 1 /* offset */, &name);
		assert!(opt_dirent.is_none());
	}

	// Adding a dirent works if there's enough capacity.
	{
		let node_id = node::NodeId::new(100).unwrap();
		let name = CString::new("foobar").unwrap();
		let dirent = resp.push(node_id, 1 /* offset */, &name).unwrap();

		assert_eq!(dirent.offset(), 1);
		assert_eq!(dirent.node_kind(), node::NodeKind::UNKNOWN);

		let mut dirent = dirent;
		assert!(dirent.node_mut().is_some());
	}

	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_direntplus>()
					+ 8) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_direntplus {
				entry_out: fuse_kernel::fuse_entry_out {
					nodeid: 100,
					attr: fuse_kernel::fuse_attr {
						ino: 100,
						..Default::default()
					},
					..Default::default()
				},
				dirent: fuse_kernel::fuse_dirent {
					ino: 100,
					off: 1,
					namelen: 6,
					r#type: node::NodeKind::UNKNOWN.raw(),
					name: [0u8; 0],
				},
			})
			.push_bytes(b"foobar\0\0")
			.build()
	);
}

#[test]
fn dirent_fields() {
	let mut fuse_dirent = fuse_kernel::fuse_dirent {
		ino: 100,
		off: 1,
		namelen: 6,
		r#type: node::NodeKind::UNKNOWN.raw(),
		name: [0u8; 0],
	};

	{
		let mut dirent = Dirent {
			node_id: node::NodeId::new(100).unwrap(),
			node: None,
			dirent: &mut fuse_dirent,
			name: b"foobar\0\0",
		};

		assert!(dirent.node_mut().is_none());

		dirent.set_node_id(node::NodeId::new(200).unwrap());
		dirent.set_offset(2);
		dirent.set_node_kind(node::NodeKind::REG);

		assert_eq!(dirent.node_id().get(), 200);
		assert_eq!(dirent.offset(), 2);
		assert_eq!(dirent.node_kind(), node::NodeKind::REG);
	}

	// READDIR mode: when `Dirent::node` is None, the mutator methods only
	// adjust the raw dirent.
	assert_eq!(fuse_dirent.ino, 200);
	assert_eq!(fuse_dirent.off, 2);
	assert_eq!(fuse_dirent.r#type, node::NodeKind::REG.raw());
}

#[test]
fn direntplus_without_node() {
	let mut fuse_entry_out = fuse_kernel::fuse_entry_out {
		..Default::default()
	};
	let mut fuse_dirent = fuse_kernel::fuse_dirent {
		ino: 100,
		off: 1,
		namelen: 6,
		r#type: node::NodeKind::UNKNOWN.raw(),
		name: [0u8; 0],
	};

	{
		let mut dirent = Dirent {
			node_id: node::NodeId::new(100).unwrap(),
			node: Some(node::Node::new_ref_mut(&mut fuse_entry_out)),
			dirent: &mut fuse_dirent,
			name: b"foobar\0\0",
		};

		dirent.set_node_id(node::NodeId::new(200).unwrap());
		dirent.set_offset(2);
		dirent.set_node_kind(node::NodeKind::REG);

		assert_eq!(dirent.node_id().get(), 200);
		assert_eq!(dirent.offset(), 2);
		assert_eq!(dirent.node_kind(), node::NodeKind::REG);
	}

	// READDIRPLUS light mode: when `Dirent::node` is set but never explicitly
	// modified, the mutator methods only adjust the raw dirent.
	//
	// The internal node data is skipped if `fuse_entry_out::nodeid` is zero.
	assert_eq!(fuse_dirent.ino, 200);
	assert_eq!(fuse_dirent.off, 2);
	assert_eq!(fuse_dirent.r#type, node::NodeKind::REG.raw());
	assert_eq!(fuse_entry_out.nodeid, 0);
	assert_eq!(fuse_entry_out.attr.mode, 0);
}

#[test]
fn direntplus_with_node() {
	let mut fuse_entry_out = fuse_kernel::fuse_entry_out {
		..Default::default()
	};
	let mut fuse_dirent = fuse_kernel::fuse_dirent {
		ino: 100,
		off: 1,
		namelen: 6,
		r#type: node::NodeKind::UNKNOWN.raw(),
		name: [0u8; 0],
	};

	{
		let mut dirent = Dirent {
			node_id: node::NodeId::new(100).unwrap(),
			node: Some(node::Node::new_ref_mut(&mut fuse_entry_out)),
			dirent: &mut fuse_dirent,
			name: b"foobar\0\0",
		};

		dirent.node_mut().unwrap();

		dirent.set_node_id(node::NodeId::new(200).unwrap());
		dirent.set_offset(2);
		dirent.set_node_kind(node::NodeKind::REG);

		assert_eq!(dirent.node_id().get(), 200);
		assert_eq!(dirent.offset(), 2);
		assert_eq!(dirent.node_kind(), node::NodeKind::REG);
	}

	// READDIRPLUS full mode: when `Dirent::node` is set and explicitly
	// modified, the mutator methods propagate dirent fields into the Node.
	assert_eq!(fuse_dirent.ino, 200);
	assert_eq!(fuse_dirent.off, 2);
	assert_eq!(fuse_dirent.r#type, node::NodeKind::REG.raw());
	assert_eq!(fuse_entry_out.nodeid, 200);
	assert_eq!(fuse_entry_out.attr.mode, 0o100000 /* S_IFREG */);

	{
		let mut dirent = Dirent {
			node_id: node::NodeId::new(100).unwrap(),
			node: Some(node::Node::new_ref_mut(&mut fuse_entry_out)),
			dirent: &mut fuse_dirent,
			name: b"foobar\0\0",
		};

		let node = dirent.node_mut().unwrap();

		node.set_id(node::NodeId::new(300).unwrap());
		node.set_kind(node::NodeKind::CHR);

		assert_eq!(dirent.node_id().get(), 300);
		assert_eq!(dirent.node_kind(), node::NodeKind::CHR);
	}

	// READDIRPLUS full mode: when shared properties are mutated by the Node
	// instead of the Dirent, it still looks to the user like a unified
	// view of the filesystem.
	assert_eq!(fuse_dirent.ino, 300);
	assert_eq!(fuse_dirent.r#type, node::NodeKind::CHR.raw());
	assert_eq!(fuse_entry_out.nodeid, 300);
	assert_eq!(fuse_entry_out.attr.mode, 0o20000 /* S_IFCHR */);
}

#[test]
fn dirent_debug() {
	let mut fuse_entry_out = fuse_kernel::fuse_entry_out {
		..Default::default()
	};
	let mut fuse_dirent = fuse_kernel::fuse_dirent {
		ino: 100,
		off: 1,
		namelen: 6,
		r#type: node::NodeKind::UNKNOWN.raw(),
		name: [0u8; 0],
	};
	let dirent = Dirent {
		node_id: node::NodeId::new(100).unwrap(),
		node: Some(node::Node::new_ref_mut(&mut fuse_entry_out)),
		dirent: &mut fuse_dirent,
		name: b"foobar\0\0",
	};

	// TODO: verify output
	assert!(format!("{:#?}", &dirent).len() > 0);
}
