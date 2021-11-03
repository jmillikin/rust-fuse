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
use std::{ffi, panic};

use fuse::server::basic;
use interop_testutil::{diff_str, errno, fuse_interop_test, path_cstr};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl interop_testutil::TestFS for TestFS {}

type S = fuse::os::unix::DevFuse;

impl basic::FuseHandlers<S> for TestFS {
	fn lookup(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::LookupRequest,
		send_reply: impl basic::SendReply<S>,
	) -> basic::SendResult<fuse::LookupResponse, std::io::Error> {
		if request.parent_id() != fuse::ROOT_ID {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}
		if request.name() != fuse::NodeName::from_bytes(b"xattrs.txt").unwrap()
		{
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		send_reply.ok(&resp)
	}

	fn getxattr(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::GetxattrRequest,
		send_reply: impl basic::SendReply<S>,
	) -> basic::SendResult<fuse::GetxattrResponse, std::io::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let xattr_small =
			fuse::XattrName::from_bytes(b"user.xattr_small").unwrap();
		let xattr_toobig =
			fuse::XattrName::from_bytes(b"user.xattr_toobig").unwrap();

		if request.name() == xattr_small {
			let mut resp = fuse::GetxattrResponse::new(request.size());
			return match resp.try_set_value(b"small xattr value") {
				Ok(_) => send_reply.ok(&resp),
				Err(_) => {
					// TODO: error should either have enough public info to let the caller
					// return an appropriate error code, or ERANGE should be handled by
					// the response dispatcher.
					send_reply.err(fuse::ErrorCode::ERANGE)
				},
			};
		}

		if request.name() == xattr_toobig {
			return send_reply.err(fuse::ErrorCode::E2BIG);
		}

		send_reply.err(fuse::ErrorCode::ENOATTR)
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
			let xattr_name = ffi::CString::new("user.xattr_small").unwrap();
			libc::getxattr(
				path.as_ptr(),
				xattr_name.as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("xattr_small").unwrap();
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				xattr_name.as_ptr(),
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
			let xattr_name = ffi::CString::new("user.xattr_small").unwrap();
			libc::getxattr(
				path.as_ptr(),
				xattr_name.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("xattr_small").unwrap();
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				xattr_name.as_ptr(),
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
			let xattr_name = ffi::CString::new("user.xattr_noexist").unwrap();
			libc::getxattr(
				path.as_ptr(),
				xattr_name.as_ptr(),
				std::ptr::null_mut(),
				0,
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("xattr_noexist").unwrap();
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				xattr_name.as_ptr(),
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
			let xattr_name = ffi::CString::new("user.xattr_small").unwrap();
			libc::getxattr(
				path.as_ptr(),
				xattr_name.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("xattr_small").unwrap();
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				xattr_name.as_ptr(),
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
			let xattr_name = ffi::CString::new("user.xattr_toobig").unwrap();
			libc::getxattr(
				path.as_ptr(),
				xattr_name.as_ptr(),
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("xattr_toobig").unwrap();
			libc::extattr_get_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				xattr_name.as_ptr(),
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
