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
	errno,
	fuse_interop_test,
	libc_errno,
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

		let mut attr = fuse::Attributes::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn getxattr(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::GetxattrRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		if request.name() == c"user.xattr_small" {
			let value = b"small xattr value";

			match request.size() {
				None => {
					let mut reply = fuse::kernel::fuse_getxattr_out::new();
					reply.size = value.len() as u32;
					return send_reply.ok(&reply).unwrap();
				},
				Some(request_size) => {
					if value.len() > request_size.get() {
						return send_reply.err(OsError(errno::ERANGE)).unwrap();
					}
				},
			};

			return send_reply.ok_buf(value).unwrap();
		}

		if request.name() == c"user.xattr_toobig" {
			return send_reply.err(OsError::XATTR_TOO_BIG).unwrap();
		}

		send_reply.err(OsError::XATTR_NOT_FOUND).unwrap();
	}
}

fn getxattr_test(
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
fn getxattr_query_size() {
	let requests = getxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		#[cfg(target_os = "linux")]
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				c"user.xattr_small".as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				c"xattr_small".as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		assert_eq!(rc, 17);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetxattrRequest {
    node_id: 2,
    size: None,
    name: "user.xattr_small",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getxattr_small() {
	let requests = getxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		let mut value = [0u8; 20];

		#[cfg(target_os = "linux")]
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				c"user.xattr_small".as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				c"xattr_small".as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		assert_eq!(rc, 17);
		assert_eq!(&value, b"small xattr value\0\0\0")
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetxattrRequest {
    node_id: 2,
    size: Some(20),
    name: "user.xattr_small",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getxattr_noexist() {
	let requests = getxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		#[cfg(target_os = "linux")]
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				c"user.xattr_noexist".as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				c"xattr_noexist".as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		assert_eq!(rc, -1);
		#[allow(deprecated)]
		const ENOATTR: i32 = libc::ENOATTR;
		assert_eq!(libc_errno(), ENOATTR);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetxattrRequest {
    node_id: 2,
    size: None,
    name: "user.xattr_noexist",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getxattr_buffer_too_small() {
	let requests = getxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		let mut value = [0u8; 1];

		#[cfg(target_os = "linux")]
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				c"user.xattr_small".as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				c"xattr_small".as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::ERANGE);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetxattrRequest {
    node_id: 2,
    size: Some(1),
    name: "user.xattr_small",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getxattr_oversize_xattr() {
	let requests = getxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		let mut value = [0u8; 32];

		#[cfg(target_os = "linux")]
		let rc = unsafe {
			libc::getxattr(
				path.as_ptr(),
				c"user.xattr_toobig".as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				c"xattr_toobig".as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::E2BIG);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetxattrRequest {
    node_id: 2,
    size: Some(32),
    name: "user.xattr_toobig",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
