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

use fuse::server;
use fuse::server::FuseRequest;

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	path_cstr,
	OsError,
};

struct TestFS {
	requests: mpsc::Sender<String>,
}

struct TestHandlers<'a, S> {
	fs: &'a TestFS,
	conn: &'a server::FuseConnection<S>,
}

impl interop_testutil::TestFS for TestFS {
	fn dispatch_request(
		&self,
		conn: &server::FuseConnection<interop_testutil::DevFuse>,
		request: FuseRequest<'_>,
	) {
		use fuse::server::FuseHandlers;
		(TestHandlers{fs: self, conn}).dispatch(request);
	}
}

impl<'a, S> server::FuseHandlers for TestHandlers<'a, S>
where
	S: server::FuseSocket,
	S::Error: core::fmt::Debug,
{
	fn unimplemented(&self, request: FuseRequest<'_>) {
		self.conn.reply(request.id()).err(OsError::UNIMPLEMENTED).unwrap();
	}

	fn lookup(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::LookupRequest::try_from(request).unwrap();

		if !request.parent_id().is_root() {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}
		if request.name() != "xattrs.txt" {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn setxattr(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::SetxattrRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();
		send_reply.ok_empty().unwrap();
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
    value: "some\x00value",
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
    value: "some\x00value",
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
    value: "some\x00value",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
