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

use interop_testutil::{diff_str, errno, interop_test, path_cstr};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl fuse::FuseHandlers for TestFS {
	fn lookup(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::LookupRequest,
		respond: impl for<'a> fuse::Respond<fuse::LookupResponse<'a>>,
	) {
		if request.parent_id() != fuse::ROOT_ID {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
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
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(node_id);
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		respond.ok(&resp);
	}

	fn listxattr(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ListxattrRequest,
		respond: impl for<'a> fuse::Respond<fuse::ListxattrResponse<'a>>,
	) {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let xattr_small = fuse::XattrName::from_bytes(b"xattr_small").unwrap();
		let xattr_toobig =
			fuse::XattrName::from_bytes(b"xattr_toobig").unwrap();

		let mut resp = match request.size() {
			None => fuse::ListxattrResponse::without_capacity(),
			Some(max_size) => {
				fuse::ListxattrResponse::with_max_size(max_size.into())
			},
		};

		if request.node_id() == fuse::NodeId::new(3).unwrap() {
			respond.err(fuse::ErrorCode::E2BIG);
			return;
		}

		if let Err(_) = resp.try_add_name(xattr_small) {
			respond.err(fuse::ErrorCode::ERANGE);
			return;
		}
		if let Err(_) = resp.try_add_name(xattr_toobig) {
			respond.err(fuse::ErrorCode::ERANGE);
			return;
		}
		respond.ok(&resp);
	}
}

fn listxattr_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	interop_test(fs, test_fn);
	request_recv.iter().collect()
}

#[test]
fn listxattr_query_size() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let rc =
			unsafe { libc::listxattr(path.as_ptr(), std::ptr::null_mut(), 0) };
		assert_eq!(rc, 25);
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
fn listxattr() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let mut name_list = [0u8; 30];

		let rc = unsafe {
			libc::listxattr(
				path.as_ptr(),
				name_list.as_mut_ptr() as *mut i8,
				name_list.len(),
			)
		};
		assert_eq!(rc, 25);
		assert_eq!(&name_list, b"xattr_small\0xattr_toobig\0\0\0\0\0\0");
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ListxattrRequest {
    node_id: 2,
    size: Some(30),
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn listxattr_buffer_too_small() {
	let requests = listxattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));
		let mut name_list = [0i8; 1];

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
    size: Some(1),
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
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
