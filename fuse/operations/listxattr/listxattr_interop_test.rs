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

use std::panic;
use std::sync::mpsc;

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

		let node_id;
		if request.name() == "xattrs.txt" {
			node_id = fuse::NodeId::new(2).unwrap();
		} else if request.name() == "xattrs_toobig.txt" {
			node_id = fuse::NodeId::new(3).unwrap();
		} else {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::Attributes::new(node_id);
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn listxattr(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ListxattrRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		if request.node_id() == fuse::NodeId::new(3).unwrap() {
			return send_reply.err(OsError(errno::E2BIG)).unwrap();
		}

		let xattr_small = fuse::XattrName::new("user.xattr_small").unwrap();
		let xattr_toobig = fuse::XattrName::new("user.xattr_toobig").unwrap();

		let buf_size = match request.size() {
			None => {
				let mut need_size = xattr_small.size();
				need_size += xattr_toobig.size();
				let mut reply = fuse::kernel::fuse_getxattr_out::new();
				reply.size = need_size as u32;
				return send_reply.ok(&reply).unwrap();
			},
			Some(request_size) => request_size,
		};

		let mut buf = vec![0u8; buf_size.get()];
		let mut names = server::ListxattrNamesWriter::new(&mut buf);

		if names.try_push(xattr_small).is_err() {
			return send_reply.err(OsError(errno::ERANGE)).unwrap();
		}
		if names.try_push(xattr_toobig).is_err() {
			return send_reply.err(OsError(errno::ERANGE)).unwrap();
		}

		send_reply.ok(&names.into_names()).unwrap();
	}
}

fn listxattr_test(
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
#[cfg(target_os = "linux")]
fn listxattr_query_size() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		let rc = unsafe {
			libc::listxattr(
				path.as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		assert_eq!(rc, 35);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "freebsd")]
fn extattr_list_query_size() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		let rc = unsafe {
			libc::extattr_list_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				std::ptr::null_mut(),
				0,
			)
		};

		assert_eq!(rc, 25);
	});
	assert_eq!(requests.len(), 2);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: Some(35),
}"#;
	if let Some(diff) = diff_str(expect, &requests[1]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn listxattr() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let mut name_list = [0u8; 40];

		let rc = unsafe {
			libc::listxattr(
				path.as_ptr(),
				name_list.as_mut_ptr() as *mut i8,
				name_list.len(),
			)
		};
		assert_eq!(rc, 35);
		assert_eq!(
			&name_list,
			b"user.xattr_small\0user.xattr_toobig\0\0\0\0\0\0"
		);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: Some(40),
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "freebsd")]
fn extattr_list() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let mut name_list = [0u8; 30];

		let rc = unsafe {
			libc::extattr_list_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				name_list.as_mut_ptr() as *mut libc::c_void,
				name_list.len(),
			)
		};

		assert_eq!(rc, 25);
		assert_eq!(&name_list, b"\x0Bxattr_small\x0Cxattr_toobig\0\0\0\0\0");
	});
	assert_eq!(requests.len(), 2);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: Some(35),
}"#;
	if let Some(diff) = diff_str(expect, &requests[1]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn listxattr_buffer_too_small() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let mut name_list = [0i8; 5];

		let rc = unsafe {
			libc::listxattr(
				path.as_ptr(),
				name_list.as_mut_ptr(),
				name_list.len(),
			)
		};

		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::ERANGE);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: Some(5),
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "freebsd")]
fn extattr_list_buffer_too_small() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let mut name_list = [0u8; 5];

		let rc = unsafe {
			libc::extattr_list_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				name_list.as_mut_ptr() as *mut libc::c_void,
				name_list.len(),
			)
		};

		assert_eq!(rc, 5);
		assert_eq!(&name_list, b"\x0Bxatt");
	});
	assert_eq!(requests.len(), 2);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: Some(35),
}"#;
	if let Some(diff) = diff_str(expect, &requests[1]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn listxattr_oversize_name_list() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs_toobig.txt"));
		let mut name_list = [0i8; 30];

		let rc = unsafe {
			libc::listxattr(
				path.as_ptr(),
				name_list.as_mut_ptr(),
				name_list.len(),
			)
		};

		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::E2BIG);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ListxattrRequest {
    node_id: 3,
    size: Some(30),
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "freebsd")]
fn extattr_list_oversize_name_list() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs_toobig.txt"));
		let mut name_list = [0i8; 30];

		let rc = unsafe {
			libc::extattr_list_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				name_list.as_mut_ptr() as *mut libc::c_void,
				name_list.len(),
			)
		};

		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::E2BIG);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ListxattrRequest {
    node_id: 3,
    size: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
