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

mod fuse {
	pub use ::fuse::*;
	pub use ::fuse::io::*;
	pub use ::fuse::protocol::*;
	pub use ::fuse::server::basic::*;
}

use interop_testutil::{diff_str, errno, fuse_interop_test, path_cstr};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl interop_testutil::TestFS for TestFS {}

impl<S: fuse::OutputStream> fuse::FuseHandlers<S> for TestFS {
	fn lookup(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::LookupRequest,
		send_reply: impl fuse::SendReply<S>,
	) -> fuse::SendResult<fuse::LookupResponse, S::Error> {
		if request.parent_id() != fuse::ROOT_ID {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let node_id;
		if request.name() == fuse::NodeName::from_bytes(b"xattrs.txt").unwrap()
		{
			node_id = fuse::NodeId::new(2).unwrap();
		} else if request.name()
			== fuse::NodeName::from_bytes(b"xattrs_toobig.txt").unwrap()
		{
			node_id = fuse::NodeId::new(3).unwrap();
		} else {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(node_id);
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		send_reply.ok(&resp)
	}

	fn listxattr(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ListxattrRequest,
		send_reply: impl fuse::SendReply<S>,
	) -> fuse::SendResult<fuse::ListxattrResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let xattr_small =
			fuse::XattrName::from_bytes(b"user.xattr_small").unwrap();
		let xattr_toobig =
			fuse::XattrName::from_bytes(b"user.xattr_toobig").unwrap();

		let mut resp = match request.size() {
			None => fuse::ListxattrResponse::without_capacity(),
			Some(max_size) => {
				fuse::ListxattrResponse::with_max_size(max_size.into())
			},
		};

		if request.node_id() == fuse::NodeId::new(3).unwrap() {
			return send_reply.err(fuse::ErrorCode::E2BIG);
		}

		if let Err(_) = resp.try_add_name(xattr_small) {
			return send_reply.err(fuse::ErrorCode::ERANGE);
		}
		if let Err(_) = resp.try_add_name(xattr_toobig) {
			return send_reply.err(fuse::ErrorCode::ERANGE);
		}
		send_reply.ok(&resp)
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
		assert_eq!(errno(), libc::ERANGE);
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
		assert_eq!(errno(), libc::E2BIG);
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
		assert_eq!(errno(), libc::E2BIG);
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
