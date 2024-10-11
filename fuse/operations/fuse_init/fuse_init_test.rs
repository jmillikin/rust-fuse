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
use fuse::server::FuseInitRequest;

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_opcode(kernel::fuse_opcode::FUSE_INIT)
		.push_sized(&7u32) // fuse_init_in::major
		.push_sized(&1u32) // fuse_init_in::minor
		.build_aligned();
	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 1);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(req.flags(), fuse::FuseInitFlags::new());
}

#[test]
fn request_v7p6() {
	let buf = MessageBuilder::new()
		.set_opcode(kernel::fuse_opcode::FUSE_INIT)
		.push_sized(&7u32) // fuse_init_in::major
		.push_sized(&6u32) // fuse_init_in::minor
		.push_sized(&9u32) // fuse_init_in::max_readahead
		.push_sized(&kernel::FUSE_ASYNC_READ) // fuse_init_in::flags
		.build_aligned();
	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 6);
	assert_eq!(req.max_readahead(), 9);
	assert_eq!(req.flags(), fuse::FuseInitFlag::ASYNC_READ);
}

#[test]
fn request_v7p36() {
	let buf = MessageBuilder::new()
		.set_opcode(kernel::fuse_opcode::FUSE_INIT)
		.push_sized(&testutil::new!(kernel::fuse_init_in {
			major: 7,
			minor: 36,
			max_readahead: 9,
			flags: kernel::FUSE_ASYNC_READ,
			flags2: (kernel::FUSE_HAS_INODE_DAX >> 32) as u32,
		}))
		.build_aligned();
	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 36);
	assert_eq!(req.max_readahead(), 9);
	assert_eq!(
		req.flags(),
		fuse::FuseInitFlag::ASYNC_READ | fuse::FuseInitFlag::HAS_INODE_DAX,
	);
}

#[test]
fn request_major_mismatch() {
	let buf = MessageBuilder::new()
		.set_opcode(kernel::fuse_opcode::FUSE_INIT)
		.push_sized(&testutil::new!(kernel::fuse_init_in {
			major: 0xFF,
			minor: 0xFF,
			max_readahead: 0xFF,
			flags: 0xFFFFFFFF,
			flags2: 0xFFFFFFFF,
		}))
		.build_aligned();
	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 0xFF);
	assert_eq!(req.version().minor(), 0xFF);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(req.flags(), fuse::FuseInitFlags::new());
}

#[test]
fn init_flags() {
	let buf = MessageBuilder::new()
		.set_opcode(kernel::fuse_opcode::FUSE_INIT)
		.push_sized(&testutil::new!(kernel::fuse_init_in {
			major: kernel::FUSE_KERNEL_VERSION,
			minor: kernel::FUSE_KERNEL_MINOR_VERSION,
			flags: 0x3,
			flags2: 0x3 | (1u32 << 31),
		}))
		.build_aligned();
	let request = decode_request!(FuseInitRequest, buf);

	// Flag sets render as a struct, with unknown flags falling back
	// to hex.
	assert_eq!(
		format!("{:#?}", request.flags()),
		concat!(
			"FuseInitFlags {\n",
			"    ASYNC_READ,\n",
			"    POSIX_LOCKS,\n",
			"    SECURITY_CTX,\n",
			"    HAS_INODE_DAX,\n",
			"    0x8000000000000000,\n",
			"}",
		),
	);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_opcode(kernel::fuse_opcode::FUSE_INIT)
		.push_sized(&testutil::new!(kernel::fuse_init_in {
			major: 7,
			minor: 6,
			max_readahead: 4096,
			flags: 0x1,
		}))
		.build_aligned();
	let request = decode_request!(FuseInitRequest, buf);

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FuseInitRequest {\n",
			"    version: Version {\n",
			"        major: 7,\n",
			"        minor: 6,\n",
			"    },\n",
			"    max_readahead: 4096,\n",
			"    flags: FuseInitFlags {\n",
			"        ASYNC_READ,\n",
			"    },\n",
			"}",
		),
	);
}
