// Copyright 2021 John Millikin and the rust-fuse contributors.
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

use std::sync::mpsc;
use std::panic;

use fuse::server::fuse_rpc;
use fuse::server::prelude::*;

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	path_cstr,
	ErrorCode,
};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl interop_testutil::TestFS for TestFS {}

impl<S: FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &LookupRequest,
	) -> fuse_rpc::SendResult<LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "xattrs.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = fuse::Attributes::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn setxattr(
		&self,
		call: fuse_rpc::Call<S>,
		request: &SetxattrRequest,
	) -> fuse_rpc::SendResult<SetxattrResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();
		let resp = SetxattrResponse::new();
		call.respond_ok(&resp)
	}
}

fn setxattr_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

#[test]
fn setxattr() {
	let requests = setxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let xattr_value = b"some\x00value";

		#[cfg(target_os = "linux")]
		{
			let rc = unsafe {
				libc::setxattr(
					path.as_ptr(),
					c"user.xattr_name".as_ptr(),
					xattr_value.as_ptr() as *const libc::c_void,
					xattr_value.len(),
					0,
				)
			};
			assert_eq!(rc, 0);
		}

		#[cfg(target_os = "freebsd")]
		{
			let rc = unsafe {
				libc::extattr_set_file(
					path.as_ptr(),
					libc::EXTATTR_NAMESPACE_USER,
					c"xattr_name".as_ptr(),
					xattr_value.as_ptr() as *const libc::c_void,
					xattr_value.len(),
				)
			};
			assert_eq!(rc, xattr_value.len() as isize);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetxattrRequest {
    node_id: 2,
    name: "user.xattr_name",
    flags: SetxattrRequestFlags {},
    setxattr_flags: 0x00000000,
    value: [
        115,
        111,
        109,
        101,
        0,
        118,
        97,
        108,
        117,
        101,
    ],
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn setxattr_flag_create() {
	let requests = setxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let xattr_value = b"some\x00value";

		let rc = unsafe {
			libc::setxattr(
				path.as_ptr(),
				c"user.xattr_name".as_ptr(),
				xattr_value.as_ptr() as *const libc::c_void,
				xattr_value.len(),
				libc::XATTR_CREATE,
			)
		};
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetxattrRequest {
    node_id: 2,
    name: "user.xattr_name",
    flags: SetxattrRequestFlags {},
    setxattr_flags: 0x00000001,
    value: [
        115,
        111,
        109,
        101,
        0,
        118,
        97,
        108,
        117,
        101,
    ],
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn setxattr_flag_replace() {
	let requests = setxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let xattr_value = b"some\x00value";

		let rc = unsafe {
			libc::setxattr(
				path.as_ptr(),
				c"user.xattr_name".as_ptr(),
				xattr_value.as_ptr() as *const libc::c_void,
				xattr_value.len(),
				libc::XATTR_REPLACE,
			)
		};
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetxattrRequest {
    node_id: 2,
    name: "user.xattr_name",
    flags: SetxattrRequestFlags {},
    setxattr_flags: 0x00000002,
    value: [
        115,
        111,
        109,
        101,
        0,
        118,
        97,
        108,
        117,
        101,
    ],
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
