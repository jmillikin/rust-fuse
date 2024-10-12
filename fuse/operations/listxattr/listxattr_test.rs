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

use core::num;

use fuse::kernel;
use fuse::server::{
	ListxattrNamesWriter,
	ListxattrRequest,
};

use fuse_testutil as testutil;
use fuse_testutil::{decode_request, MessageBuilder};

#[test]
fn request_sized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LISTXATTR;
			h.nodeid = 123;
		})
		.push_sized(&testutil::new!(kernel::fuse_getxattr_in {
			size: 10,
		}))
		.build_aligned();

	let req = decode_request!(ListxattrRequest, buf);

	assert_eq!(req.size(), Some(num::NonZeroUsize::new(10).unwrap()));
}

#[test]
fn request_unsized() {
	let buf = MessageBuilder::new()
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LISTXATTR;
			h.nodeid = 123;
		})
		.push_sized(&kernel::fuse_getxattr_in::new())
		.build_aligned();

	let req = decode_request!(ListxattrRequest, buf);

	assert_eq!(req.size(), None);
}

#[test]
fn request_impl_debug() {
	let buf;
	let request = fuse_testutil::build_request!(buf, ListxattrRequest, {
		.set_header(|h| {
			h.opcode = kernel::fuse_opcode::FUSE_LISTXATTR;
			h.nodeid = kernel::FUSE_ROOT_ID;
		})
		.push_sized(&testutil::new!(kernel::fuse_getxattr_in {
			size: 11,
		}))
	});

	assert_eq!(
		format!("{:#?}", request),
		concat!(
			"ListxattrRequest {\n",
			"    node_id: 1,\n",
			"    size: Some(11),\n",
			"}",
		),
	);
}

#[test]
fn listxattr_names() {
	let mut buf = [0u8; 10];
	let mut names = ListxattrNamesWriter::new(&mut buf);
	assert_eq!(names.capacity(), 10);

	// response must fit in provided buffer
	{
		assert!(names.try_push(c"12345678901").is_err());
	}

	// xattr names are NUL-terminated, so two 3-byte names requires 8 bytes
	// of buffer space.
	names.try_push(c"123").unwrap();
	names.try_push(c"456").unwrap();
	assert_eq!(names.position(), 8);

	let names = names.into_names();
	assert_eq!(names.as_bytes(), b"123\x00456\x00")
}
