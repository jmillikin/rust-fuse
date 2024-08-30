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

use fuse::node;
use fuse::server::fuse_rpc;
use fuse::server::prelude::*;
use fuse::xattr;

use interop_testutil::{
	diff_str,
	errno,
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

		let mut attr = node::Attributes::new(node::Id::new(2).unwrap());
		attr.set_mode(node::Mode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = node::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn getxattr(
		&self,
		call: fuse_rpc::Call<S>,
		request: &GetxattrRequest,
	) -> fuse_rpc::SendResult<GetxattrResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let xattr_small = xattr::Name::new("user.xattr_small").unwrap();
		let xattr_toobig = xattr::Name::new("user.xattr_toobig").unwrap();

		if request.name() == xattr_small {
			let value = xattr::Value::new(b"small xattr value").unwrap();

			match request.size() {
				None => {
					let resp = GetxattrResponse::with_value_size(value.size());
					return call.respond_ok(&resp);
				},
				Some(request_size) => {
					if value.size() > request_size.get() {
						return call.respond_err(ErrorCode::ERANGE);
					}
				},
			};

			let resp = GetxattrResponse::with_value(value);
			return call.respond_ok(&resp);
		}

		if request.name() == xattr_toobig {
			return call.respond_err(ErrorCode::E2BIG);
		}

		#[cfg(target_os = "linux")]
		let err = ErrorCode::ENODATA;

		#[cfg(target_os = "freebsd")]
		let err = ErrorCode::ENOATTR;

		call.respond_err(err)
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
		assert_eq!(errno(), ENOATTR);
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
		assert_eq!(errno(), libc::ERANGE);
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
		assert_eq!(errno(), libc::E2BIG);
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
