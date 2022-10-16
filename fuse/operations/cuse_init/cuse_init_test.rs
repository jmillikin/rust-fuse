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
use fuse::cuse;
use fuse::operations::cuse_init::{
	CuseInitFlag,
	CuseInitRequest,
	CuseInitResponse,
};
use fuse::server;

use fuse_testutil::MessageBuilder;

#[test]
fn request() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::CUSE_INIT)
		.push_sized(&fuse_kernel::cuse_init_in {
			major: 7,
			minor: 6,
			unused: 0,
			flags: fuse_kernel::CUSE_UNRESTRICTED_IOCTL,
		})
		.build_aligned();

	let req = CuseInitRequest::from_request(
		server::Request::new(buf.as_aligned_slice()).unwrap(),
	).unwrap();

	assert_eq!(req.version().major(), 7);
	assert_eq!(req.version().minor(), 6);
	assert_eq!(req.flags(), CuseInitFlag::UNRESTRICTED_IOCTL);
}

#[test]
fn request_impl_debug() {
	let buf = MessageBuilder::new()
		.set_opcode(fuse_kernel::CUSE_INIT)
		.push_sized(&fuse_kernel::cuse_init_in {
			major: 7,
			minor: 1,
			unused: 0,
			flags: 0x1,
		})
		.build_aligned();

	let request = CuseInitRequest::from_request(
		server::Request::new(buf.as_aligned_slice()).unwrap(),
	).unwrap();

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"CuseInitRequest {\n",
			"    version: Version {\n",
			"        major: 7,\n",
			"        minor: 1,\n",
			"    },\n",
			"    flags: CuseInitFlags {\n",
			"        UNRESTRICTED_IOCTL,\n",
			"    },\n",
			"}",
		),
	);
}

#[test]
fn response() {
	let device_name = cuse::DeviceName::new("test-device").unwrap();
	let mut resp = CuseInitResponse::new(device_name);
	resp.set_version(Version::new(7, 23));
	resp.set_max_write(4096);
	resp.update_flags(|flags| {
		flags.set(CuseInitFlag::UNRESTRICTED_IOCTL);
	});

	let request_id = core::num::NonZeroU64::new(0xAABBCCDD).unwrap();
	let mut header = fuse::ResponseHeader::new(request_id);
	let encoded = fuse::io::SendBuf::from(resp.to_response(&mut header))
		.to_vec()
		.unwrap();

	assert_eq!(
		encoded,
		MessageBuilder::new()
			.push_sized(&fuse_kernel::fuse_out_header {
				len: (size_of::<fuse_kernel::fuse_out_header>()
					+ size_of::<fuse_kernel::cuse_init_out>()
					+ b"DEVNAME=test-device\x00".len()) as u32,
				error: 0,
				unique: 0xAABBCCDD,
			})
			.push_sized(&fuse_kernel::cuse_init_out {
				major: 7,
				minor: 23,
				unused: 0,
				flags: fuse_kernel::CUSE_UNRESTRICTED_IOCTL,
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
	let device_name = cuse::DeviceName::new("test-device").unwrap();
	let mut response = CuseInitResponse::new(device_name);
	response.set_version(fuse::Version::new(123, 456));
	response.set_max_read(4096);
	response.set_max_write(8192);
	response.set_device_number(cuse::DeviceNumber::new(10, 11));
	response.update_flags(|flags| {
		flags.set(CuseInitFlag::UNRESTRICTED_IOCTL);
	});

	assert_eq!(
		format!("{:#?}", response),
		concat!(
			"CuseInitResponse {\n",
			"    device_name: \"test-device\",\n",
			"    version: Version {\n",
			"        major: 123,\n",
			"        minor: 456,\n",
			"    },\n",
			"    flags: CuseInitFlags {\n",
			"        UNRESTRICTED_IOCTL,\n",
			"    },\n",
			"    max_read: 4096,\n",
			"    max_write: 8192,\n",
			"    device_number: DeviceNumber {\n",
			"        major: 10,\n",
			"        minor: 11,\n",
			"    },\n",
			"}",
		),
	);
}
