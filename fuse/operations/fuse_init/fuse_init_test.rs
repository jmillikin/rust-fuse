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

use fuse::Version;
use fuse::operations::fuse_init::{
	FuseInitFlag,
	FuseInitFlags,
	FuseInitRequest,
	FuseInitResponse,
};

use fuse_testutil::{decode_request, encode_response, MessageBuilder};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&7u32) // fuse_init_in::major
		.push_sized(&1u32) // fuse_init_in::minor
		.build_aligned();

	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 1);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(req.flags(), FuseInitFlags::new());
}

#[test]
fn request_v7p6() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&7u32) // fuse_init_in::major
		.push_sized(&6u32) // fuse_init_in::minor
		.push_sized(&9u32) // fuse_init_in::max_readahead
		.push_sized(&fuse_kernel::FUSE_ASYNC_READ) // fuse_init_in::flags
		.build_aligned();

	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 6);
	assert_eq!(req.max_readahead(), 9);
	assert_eq!(req.flags(), FuseInitFlag::ASYNC_READ);
}

#[test]
fn request_v7p36() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&fuse_kernel::fuse_init_in {
			major: 7,
			minor: 36,
			max_readahead: 9,
			flags: fuse_kernel::FUSE_ASYNC_READ,
			flags2: (fuse_kernel::FUSE_HAS_INODE_DAX >> 32) as u32,
			unused: [0; 11],
		})
		.build_aligned();

	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 36);
	assert_eq!(req.max_readahead(), 9);
	assert_eq!(
		req.flags(),
		FuseInitFlag::ASYNC_READ | FuseInitFlag::HAS_INODE_DAX,
	);
}

#[test]
fn request_major_mismatch() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&fuse_kernel::fuse_init_in {
			major: 0xFF,
			minor: 0xFF,
			max_readahead: 0xFF,
			flags: 0xFFFFFFFF,
			flags2: 0xFFFFFFFF,
			unused: [0xFFFFFFFF; 11],
		})
		.build_aligned();

	let req = decode_request!(FuseInitRequest, buf);

	assert_eq!(req.version().major(), 0xFF);
	assert_eq!(req.version().minor(), 0xFF);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(req.flags(), FuseInitFlags::new());
}

#[test]
fn response_v7p1() {
	let mut resp = FuseInitResponse::new();
	resp.set_version(Version::new(7, 1));
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_INIT_OUT_SIZE) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&7u32) // fuse_init_in::major
			.push_sized(&1u32) // fuse_init_in::minor
			.build()
	);
}

#[test]
fn response_v7p5() {
	let mut resp = FuseInitResponse::new();
	resp.set_version(Version::new(7, 5));
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ fuse_kernel::FUSE_COMPAT_22_INIT_OUT_SIZE) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&7u32) // fuse_init_out_v7p5::major
			.push_sized(&5u32) // fuse_init_out_v7p5::minor
			.push_sized(&[0u32; 3]) // fuse_init_out_v7p5::unused
			.push_sized(&0u32) // fuse_init_out_v7p5::max_write
			.build()
	);
}

#[test]
fn response_v7p23() {
	let mut resp = FuseInitResponse::new();
	resp.set_version(Version::new(7, 23));
	resp.set_max_readahead(4096);
	resp.mut_flags().set(FuseInitFlag::ASYNC_READ);
	resp.mut_flags().set(FuseInitFlag::HAS_INODE_DAX);
	let encoded = encode_response!(resp);

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::fuse_init_out>()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::fuse_init_out {
				major: 7,
				minor: 23,
				max_readahead: 4096,
				flags: fuse_kernel::FUSE_ASYNC_READ,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
				time_gran: 0,
				max_pages: 0,
				map_alignment: 0,
				flags2: (fuse_kernel::FUSE_HAS_INODE_DAX >> 32) as u32,
				unused: [0; 7],
			})
			.build()
	);
}

#[test]
fn init_flags() {
	let buf;
	let request = fuse_testutil::build_request!(buf, FuseInitRequest, {
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&fuse_kernel::fuse_init_in {
			major: fuse_testutil::VERSION.0,
			minor: fuse_testutil::VERSION.1,
			max_readahead: 0,
			flags: 0x3,
			flags2: 0x3 | (1u32 << 31),
			unused: [0; 11],
		})
	});

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
	let buf;
	let request = fuse_testutil::build_request!(buf, FuseInitRequest, {
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&fuse_kernel::fuse_init_in {
			major: 7,
			minor: 6,
			max_readahead: 4096,
			flags: 0x1,
			flags2: 0,
			unused: [0; 11],
		})
	});

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

#[test]
fn response_impl_debug() {
	let mut response = FuseInitResponse::new();
	response.set_max_readahead(4096);
	response.set_max_write(8192);
	response.set_max_background(10);
	response.set_congestion_threshold(11);
	response.set_time_granularity(100);
	response.mut_flags().set(FuseInitFlag::ASYNC_READ);

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"FuseInitResponse {\n",
			"    max_readahead: 4096,\n",
			"    flags: FuseInitFlags {\n",
			"        ASYNC_READ,\n",
			"    },\n",
			"    max_background: 10,\n",
			"    congestion_threshold: 11,\n",
			"    max_write: 8192,\n",
			"    time_granularity: 100,\n",
			"}",
		),
	);
}
