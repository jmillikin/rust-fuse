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

use super::{CuseInitFlags, CuseInitRequest, CuseInitResponse};

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::CUSE_INIT)
		.push_sized(&fuse_kernel::cuse_init_in {
			major: 7,
			minor: 6,
			unused: 0,
			flags: 0xFFFFFFFF,
		})
		.build_aligned();

	let req: CuseInitRequest = decode_request!(buf);

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 6);
	assert_eq!(*req.flags(), CuseInitFlags::from_bits(0xFFFFFFFF));
}

#[test]
fn request_impl_debug() {
	let version = ProtocolVersion::new(7, 1);
	let request = &CuseInitRequest {
		phantom: PhantomData,
		version: version,
		flags: CuseInitFlags::from_bits(0x1),
	};

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"CuseInitRequest {\n",
			"    version: ProtocolVersion {\n",
			"        major: 7,\n",
			"        minor: 1,\n",
			"    },\n",
			"    flags: CuseInitFlags {\n",
			"        unrestricted_ioctl: true,\n",
			"    },\n",
			"}",
		),
	);
}

fn encode_response(
	response: CuseInitResponse,
	maybe_device_name: Option<&[u8]>,
) -> Vec<u8> {
	let request_id = 0;
	let mut channel = crate::internal::testutil::FakeChannel::new();
	let encoder =
		ResponseEncoder::new(&mut channel, request_id, ProtocolVersion::LATEST);
	response
		.encode_response(encoder, maybe_device_name)
		.unwrap();
	channel.expect_write()
}

#[test]
fn response() {
	let mut resp = CuseInitResponse::new();
	resp.set_version(ProtocolVersion::new(7, 23));
	resp.set_max_write(4096);
	*resp.flags_mut() = CuseInitFlags::from_bits(0xFFFFFFFF);
	let encoded = encode_response(resp, Some(b"test-device"));

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::cuse_init_out>()
					+ b"DEVNAME=test-device\x00".len()) as u32,
				error: 0,
				unique: 0,
			})
			.push_sized(&fuse_kernel::cuse_init_out {
				major: 7,
				minor: 23,
				unused: 0,
				flags: 0xFFFFFFFF,
				max_read: 0,
				max_write: 4096,
				dev_major: 0,
				dev_minor: 0,
				spare: [0; 10],
			})
			.push_bytes(b"DEVNAME=test-device\x00")
			.build()
	);
}

#[test]
fn response_impl_debug() {
	let mut response = CuseInitResponse::new();
	response.set_max_read(4096);
	response.set_max_write(8192);
	response.set_dev_major(10);
	response.set_dev_minor(11);
	response.flags_mut().unrestricted_ioctl = true;

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"CuseInitResponse {\n",
			"    flags: CuseInitFlags {\n",
			"        unrestricted_ioctl: true,\n",
			"    },\n",
			"    max_read: 4096,\n",
			"    max_write: 8192,\n",
			"    dev_major: 10,\n",
			"    dev_minor: 11,\n",
			"}",
		),
	);
}
