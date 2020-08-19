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
use crate::internal::types::ProtocolVersion;
use crate::protocol::prelude::*;

use super::{FuseInitFlags, FuseInitRequest, FuseInitResponse};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&super::fuse_init_in_v7p1 { major: 7, minor: 1 })
		.build_aligned();

	let req: FuseInitRequest = decode_request!(buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 1);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(*req.flags(), FuseInitFlags::from_bits(0));
}

#[test]
fn request_v7p6() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&fuse_kernel::fuse_init_in {
			major: 7,
			minor: 6,
			max_readahead: 9,
			flags: 0xFFFFFFFF,
		})
		.build_aligned();

	let req: FuseInitRequest = decode_request!(buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 6);
	assert_eq!(req.max_readahead(), 9);
	assert_eq!(*req.flags(), FuseInitFlags::from_bits(0xFFFFFFFF));
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
		})
		.build_aligned();

	let req: FuseInitRequest = decode_request!(buf);

	assert_eq!(req.version().major(), 0xFF);
	assert_eq!(req.version().minor(), 0xFF);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(*req.flags(), FuseInitFlags::from_bits(0));
}

#[test]
fn response_v7p1() {
	let resp = FuseInitResponse::new(ProtocolVersion::new(7, 1));
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
			.push_sized(&super::fuse_init_out_v7p1 { major: 7, minor: 1 })
			.build()
	);
}

#[test]
fn response_v7p5() {
	let resp = FuseInitResponse::new(ProtocolVersion::new(7, 5));
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
			.push_sized(&super::fuse_init_out_v7p5 {
				major: 7,
				minor: 5,
				max_readahead: 0,
				flags: 0,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
			})
			.build()
	);
}

#[test]
fn response_v7p23() {
	let mut resp = FuseInitResponse::new(ProtocolVersion::new(7, 23));
	resp.set_max_readahead(4096);
	*resp.flags_mut() = FuseInitFlags::from_bits(0xFFFFFFFF);
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
				flags: 0xFFFFFFFF,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
				time_gran: 0,
				unused: [0; 9],
			})
			.build()
	);
}

#[test]
fn response_minor_mismatch() {
	let resp = FuseInitResponse::for_request_impl(&FuseInitRequest {
		phantom: PhantomData,
		version: ProtocolVersion::new(fuse_kernel::FUSE_KERNEL_VERSION, 0xFF),
		max_readahead: 4096,
		flags: FuseInitFlags::from_bits(0xFFFFFFFF),
	});
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
				major: fuse_kernel::FUSE_KERNEL_VERSION,
				minor: fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
				max_readahead: 4096,
				flags: 0x003F9FFF,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
				time_gran: 0,
				unused: [0; 9],
			})
			.build()
	);
}

#[test]
fn response_major_mismatch() {
	let resp = FuseInitResponse::for_request_impl(&FuseInitRequest {
		phantom: PhantomData,
		version: ProtocolVersion::new(0xFF, 0xFF),
		max_readahead: 0,
		flags: FuseInitFlags::from_bits(0),
	});
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
				major: fuse_kernel::FUSE_KERNEL_VERSION,
				minor: fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
				max_readahead: 0,
				flags: 0,
				max_background: 0,
				congestion_threshold: 0,
				max_write: 0,
				time_gran: 0,
				unused: [0; 9],
			})
			.build()
	);
}

#[test]
fn init_flags() {
	// Flag sets render as a struct, with unknown flags falling back
	// to hex.
	assert_eq!(
		format!("{:#?}", FuseInitFlags::from_bits(0x3 | (1u32 << 31))),
		concat!(
			"FuseInitFlags {\n",
			"    async_read: true,\n",
			"    posix_locks: true,\n",
			"    file_ops: false,\n",
			"    atomic_o_trunc: false,\n",
			"    export_support: false,\n",
			"    big_writes: false,\n",
			"    dont_mask: false,\n",
			"    splice_write: false,\n",
			"    splice_move: false,\n",
			"    splice_read: false,\n",
			"    flock_locks: false,\n",
			"    has_ioctl_dir: false,\n",
			"    auto_inval_data: false,\n",
			"    do_readdirplus: false,\n",
			"    readdirplus_auto: false,\n",
			"    async_dio: false,\n",
			"    writeback_cache: false,\n",
			"    no_open_support: false,\n",
			"    parallel_dirops: false,\n",
			"    handle_killpriv: false,\n",
			"    posix_acl: false,\n",
			"    abort_error: false,\n",
			"    0x80000000: true,\n",
			"}",
		),
	);
}

#[test]
fn request_impl_debug() {
	let version = ProtocolVersion::new(7, 1);
	let request = &FuseInitRequest {
		phantom: PhantomData,
		version: version,
		max_readahead: 4096,
		flags: FuseInitFlags::from_bits(0x1),
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"FuseInitRequest {\n",
			"    version: ProtocolVersion {\n",
			"        major: 7,\n",
			"        minor: 1,\n",
			"    },\n",
			"    max_readahead: 4096,\n",
			"    flags: FuseInitFlags {\n",
			"        async_read: true,\n",
			"        posix_locks: false,\n",
			"        file_ops: false,\n",
			"        atomic_o_trunc: false,\n",
			"        export_support: false,\n",
			"        big_writes: false,\n",
			"        dont_mask: false,\n",
			"        splice_write: false,\n",
			"        splice_move: false,\n",
			"        splice_read: false,\n",
			"        flock_locks: false,\n",
			"        has_ioctl_dir: false,\n",
			"        auto_inval_data: false,\n",
			"        do_readdirplus: false,\n",
			"        readdirplus_auto: false,\n",
			"        async_dio: false,\n",
			"        writeback_cache: false,\n",
			"        no_open_support: false,\n",
			"        parallel_dirops: false,\n",
			"        handle_killpriv: false,\n",
			"        posix_acl: false,\n",
			"        abort_error: false,\n",
			"    },\n",
			"}",
		),
	);
}

#[test]
fn response_impl_debug() {
	let version = ProtocolVersion::new(7, 1);
	let mut response = FuseInitResponse::new(version);
	response.set_max_readahead(4096);
	response.set_max_write(8192);
	response.set_max_background(10);
	response.set_congestion_threshold(11);
	response.set_time_granularity(100);
	response.flags_mut().async_read = true;

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"FuseInitResponse {\n",
			"    version: ProtocolVersion {\n",
			"        major: 7,\n",
			"        minor: 1,\n",
			"    },\n",
			"    max_readahead: 4096,\n",
			"    flags: FuseInitFlags {\n",
			"        async_read: true,\n",
			"        posix_locks: false,\n",
			"        file_ops: false,\n",
			"        atomic_o_trunc: false,\n",
			"        export_support: false,\n",
			"        big_writes: false,\n",
			"        dont_mask: false,\n",
			"        splice_write: false,\n",
			"        splice_move: false,\n",
			"        splice_read: false,\n",
			"        flock_locks: false,\n",
			"        has_ioctl_dir: false,\n",
			"        auto_inval_data: false,\n",
			"        do_readdirplus: false,\n",
			"        readdirplus_auto: false,\n",
			"        async_dio: false,\n",
			"        writeback_cache: false,\n",
			"        no_open_support: false,\n",
			"        parallel_dirops: false,\n",
			"        handle_killpriv: false,\n",
			"        posix_acl: false,\n",
			"        abort_error: false,\n",
			"    },\n",
			"    max_background: 10,\n",
			"    congestion_threshold: 11,\n",
			"    max_write: 8192,\n",
			"    time_granularity: 100,\n",
			"}",
		),
	);
}
