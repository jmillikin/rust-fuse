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

use fuse::node;
use fuse::operations::create::{
	CreateRequest,
	CreateRequestFlags,
	CreateResponse,
	CreateResponseFlag,
};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_CREATE;
			h.nodeid = 123;
		})
		.push_sized(&0xFFu32) // fuse_create_in::flags
		.push_sized(&0u32) // fuse_create_in::unused
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(CreateRequest, buf, {
		protocol_version: (7, 1),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
	assert_eq!(req.flags(), CreateRequestFlags::new());
	assert_eq!(req.open_flags(), 0xFF);
	assert_eq!(req.mode(), node::Mode::new(0));
	assert_eq!(req.umask(), 0);
}

#[test]
fn request_v7p12() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_CREATE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_create_in {
			flags: 0xFF,
			mode: 0xEE,
			umask: 0xDD,
			open_flags: 0, // TODO
		})
		.push_bytes(b"hello.world!\x00")
		.build_aligned();

	let req = decode_request!(CreateRequest, buf, {
		protocol_version: (7, 12),
	});

	let expect: &[u8] = b"hello.world!";
	assert_eq!(req.name(), expect);
	assert_eq!(req.flags(), CreateRequestFlags::new());
	assert_eq!(req.open_flags(), 0xFF);
	assert_eq!(req.mode(), node::Mode::new(0xEE));
	assert_eq!(req.umask(), 0xDD);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, CreateRequest, {
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_CREATE;
			h.nodeid = fuse_kernel::FUSE_ROOT_ID;
		})
		.push_sized(&fuse_kernel::fuse_create_in {
			flags: 123,
			mode: 0o100644,
			umask: 0o22,
			open_flags: 0, // TODO
		})
		.push_bytes(b"hello.world\x00")
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"CreateRequest {\n",
			"    node_id: 1,\n",
			"    name: \"hello.world\",\n",
			"    flags: CreateRequestFlags {},\n",
			"    open_flags: 0x0000007B,\n",
			"    mode: 0o100644,\n",
			"    umask: 18,\n",
			"}",
		),
	);
}

#[test]
fn response_v7p1() {
	let attr = node::Attributes::new(node::Id::new(11).unwrap());
	let mut entry = node::Entry::new(attr);
	entry.set_generation(22);

	let mut resp = CreateResponse::new(entry);
	resp.set_handle(123);
	resp.update_flags(|flags| {
		flags.set(CreateResponseFlag::DIRECT_IO);
		flags.set(CreateResponseFlag::KEEP_CACHE);
	});

	let encoded = encode_response!(resp, {
		protocol_version: (7, 1),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE
					+ size_of::<fuse_kernel::fuse_open_out>()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_entry_out {
				nodeid: 11,
				generation: 22,
				entry_valid: 0,
				attr_valid: 0,
				entry_valid_nsec: 0,
				attr_valid_nsec: 0,
				attr: fuse_kernel::fuse_attr {
					ino: 11,
					..fuse_kernel::fuse_attr::zeroed()
				}
			})
			.unpush(
				size_of::<fuse_kernel::fuse_entry_out>()
					- fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE
			)
			.push_sized(&fuse_kernel::fuse_open_out {
				fh: 123,
				open_flags: 0b11,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_v7p9() {
	let attr = node::Attributes::new(node::Id::new(11).unwrap());
	let mut entry = node::Entry::new(attr);
	entry.set_generation(22);

	let mut resp = CreateResponse::new(entry);
	resp.set_handle(123);
	resp.update_flags(|flags| {
		flags.set(CreateResponseFlag::DIRECT_IO);
		flags.set(CreateResponseFlag::KEEP_CACHE);
	});

	let encoded = encode_response!(resp, {
		protocol_version: (7, 9),
	});

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_entry_out>()
					+ size_of::<fuse_kernel::fuse_open_out>()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::fuse_entry_out {
				nodeid: 11,
				generation: 22,
				entry_valid: 0,
				attr_valid: 0,
				entry_valid_nsec: 0,
				attr_valid_nsec: 0,
				attr: fuse_kernel::fuse_attr {
					ino: 11,
					..fuse_kernel::fuse_attr::zeroed()
				}
			})
			.push_sized(&fuse_kernel::fuse_open_out {
				fh: 123,
				open_flags: 0b11,
				padding: 0,
			})
			.build()
	);
}

#[test]
fn response_impl_debug() {

	let mut attr = node::Attributes::new(node::Id::new(11).unwrap());
	attr.set_mode(node::Mode::S_IFREG | 0o644);

	let mut entry = node::Entry::new(attr);
	entry.set_generation(22);

	let mut response = CreateResponse::new(entry);
	response.set_handle(123);
	response.update_flags(|flags| {
		flags.set(CreateResponseFlag::DIRECT_IO);
		flags.set(CreateResponseFlag::KEEP_CACHE);
	});

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"CreateResponse {\n",
			"    entry: Entry {\n",
			"        generation: 22,\n",
			"        attributes: Attributes {\n",
			"            node_id: 11,\n",
			"            mode: 0o100644,\n",
			"            size: 0,\n",
			"            atime: UnixTime(0.000000000),\n",
			"            mtime: UnixTime(0.000000000),\n",
			"            ctime: UnixTime(0.000000000),\n",
			"            link_count: 0,\n",
			"            user_id: 0,\n",
			"            group_id: 0,\n",
			"            device_number: 0,\n",
			"            block_count: 0,\n",
			"            block_size: 0,\n",
			"            flags: AttributeFlags {},\n",
			"        },\n",
			"        cache_timeout: 0ns,\n",
			"        attribute_cache_timeout: 0ns,\n",
			"    },\n",
			"    handle: 123,\n",
			"    flags: CreateResponseFlags {\n",
			"        DIRECT_IO,\n",
			"        KEEP_CACHE,\n",
			"    },\n",
			"}",
		),
	);
}
