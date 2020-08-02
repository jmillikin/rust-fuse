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

use super::{FuseInitFlag, FuseInitFlags, FuseInitRequest, FuseInitResponse};

#[test]
fn request_v7p1() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::FUSE_INIT)
		.push_sized(&super::fuse_init_in_v7p1 { major: 7, minor: 1 })
		.build_aligned();

	let req: FuseInitRequest = decode_request!(buf);

	assert_eq!(req.protocol_version().major(), 7);
	assert_eq!(req.protocol_version().minor(), 1);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(req.flags(), FuseInitFlags { bits: 0 });
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

	assert_eq!(req.protocol_version().major(), 7);
	assert_eq!(req.protocol_version().minor(), 6);
	assert_eq!(req.max_readahead(), 9);
	assert_eq!(req.flags(), FuseInitFlags { bits: 0xFFFFFFFF });
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

	assert_eq!(req.protocol_version().major(), 0xFF);
	assert_eq!(req.protocol_version().minor(), 0xFF);
	assert_eq!(req.max_readahead(), 0);
	assert_eq!(req.flags(), FuseInitFlags { bits: 0 });
}

#[test]
fn response_v7p1() {
	let resp = FuseInitResponse::new(crate::ProtocolVersion::new(7, 1));
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
	let resp = FuseInitResponse::new(crate::ProtocolVersion::new(7, 5));
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
	let mut resp = FuseInitResponse::new(crate::ProtocolVersion::new(7, 23));
	resp.set_max_readahead(4096);
	resp.set_flags(FuseInitFlags { bits: 0xFFFFFFFF });
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
	let resp = FuseInitResponse::for_request(&FuseInitRequest {
		protocol_version: crate::ProtocolVersion::new(
			fuse_kernel::FUSE_KERNEL_VERSION,
			0xFF,
		),
		max_readahead: 4096,
		flags: FuseInitFlags { bits: 0xFFFFFFFF },
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
fn response_major_mismatch() {
	let resp = FuseInitResponse::for_request(&FuseInitRequest {
		protocol_version: crate::ProtocolVersion::new(0xFF, 0xFF),
		max_readahead: 0,
		flags: FuseInitFlags { bits: 0 },
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
	// Formatting known flags works.
	assert_eq!(format!("{:?}", FuseInitFlag::ASYNC_READ), "ASYNC_READ");

	// Formatting unknown flags falls back to hex.
	assert_eq!(
		format!("{:?}", FuseInitFlag { bits: 1 << 31 }),
		"0x80000000"
	);

	// Flag set renders as a list.
	assert_eq!(
		format!("{:?}", FuseInitFlags { bits: 0x3 }),
		"[ASYNC_READ, POSIX_LOCKS]"
	);

	// Flags support explicit formatting modes.
	assert_eq!(format!("{:#b}", FuseInitFlag { bits: 1 }), "0b1");
	assert_eq!(format!("{:#x}", FuseInitFlag { bits: 1 }), "0x1");
	assert_eq!(format!("{:#X}", FuseInitFlag { bits: 1 }), "0x1");

	assert_eq!(format!("{:#b}", FuseInitFlag { bits: 1 }), "0b1");
	assert_eq!(format!("{:#x}", FuseInitFlag { bits: 1 }), "0x1");
	assert_eq!(format!("{:#X}", FuseInitFlag { bits: 1 }), "0x1");
}
