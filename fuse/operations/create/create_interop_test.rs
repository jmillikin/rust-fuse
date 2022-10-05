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

use fuse::node;
use fuse::server::fuse_rpc;

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

impl<S: fuse_rpc::FuseSocket> fuse_rpc::FuseHandlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::FuseCall<S>,
		_request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		call.respond_err(ErrorCode::ENOENT)
	}

	fn create(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::CreateRequest,
	) -> fuse_rpc::FuseResult<fuse::CreateResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = fuse::CreateResponse::new();
		resp.set_handle(12345);

		let node = resp.node_mut();
		node.set_id(node::Id::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_file_type(node::Type::Regular);
		attr.set_permissions(0o644);
		attr.set_nlink(2);

		call.respond_ok(&resp)
	}
}

fn create_test(
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
fn create() {
	let requests = create_test(|root| {
		let path = path_cstr(root.join("create.txt"));

		let file_fd = unsafe { libc::creat(path.as_ptr(), 0o644) };
		assert_ne!(file_fd, -1);
		unsafe {
			libc::close(file_fd)
		};
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let flags = 33345;

	#[cfg(target_os = "freebsd")]
	let flags = 514;

	let expect = format!(
		r#"CreateRequest {{
    node_id: 1,
    name: "create.txt",
    flags: CreateRequestFlags {{}},
    open_flags: {:#010X},
    mode: 0o100644,
    umask: 18,
}}"#,
		flags
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn create_open() {
	let requests = create_test(|root| {
		let path = path_cstr(root.join("create.txt"));

		let file_fd = unsafe {
			libc::open(path.as_ptr(), libc::O_WRONLY | libc::O_CREAT, 0o644)
		};
		assert_ne!(file_fd, -1);
		unsafe {
			libc::close(file_fd)
		};
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let flags = 32833;

	#[cfg(target_os = "freebsd")]
	let flags = 514;

	let expect = format!(
		r#"CreateRequest {{
    node_id: 1,
    name: "create.txt",
    flags: CreateRequestFlags {{}},
    open_flags: {:#010X},
    mode: 0o100644,
    umask: 18,
}}"#,
		flags
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn create_exclusive() {
	let requests = create_test(|root| {
		let path = path_cstr(root.join("create.txt"));

		let file_fd = unsafe {
			libc::open(
				path.as_ptr(),
				libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
				0o644,
			)
		};
		assert_ne!(file_fd, -1);
		unsafe {
			libc::close(file_fd)
		};
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let flags = 32961;

	#[cfg(target_os = "freebsd")]
	let flags = 514;

	let expect = format!(
		r#"CreateRequest {{
    node_id: 1,
    name: "create.txt",
    flags: CreateRequestFlags {{}},
    open_flags: {:#010X},
    mode: 0o100644,
    umask: 18,
}}"#,
		flags
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
