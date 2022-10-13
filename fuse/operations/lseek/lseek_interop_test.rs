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

impl<S: fuse_rpc::FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "lseek.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = node::Attributes::new(node::Id::new(2).unwrap());
		attr.set_mode(node::Mode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = node::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = fuse::LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		_request: &fuse::OpenRequest,
	) -> fuse_rpc::FuseResult<fuse::OpenResponse, S::Error> {
		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(12345);
		call.respond_ok(&resp)
	}

	fn lseek(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::LseekRequest,
	) -> fuse_rpc::FuseResult<fuse::LseekResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = fuse::LseekResponse::new();
		resp.set_offset(4096);
		call.respond_ok(&resp)
	}
}

fn lseek_test(
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
fn lseek_set() {
	let requests = lseek_test(|root| {
		let path = path_cstr(root.join("lseek.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let lseek_rc = unsafe { libc::lseek(file_fd, 1024, libc::SEEK_SET) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(lseek_rc, 1024);
	});
	assert_eq!(requests.len(), 0);
}

#[test]
fn lseek_data() {
	let requests = lseek_test(|root| {
		let path = path_cstr(root.join("lseek.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let lseek_rc = unsafe { libc::lseek(file_fd, 1024, libc::SEEK_DATA) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(lseek_rc, 4096);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"LseekRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    whence: SEEK_DATA,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn lseek_hole() {
	let requests = lseek_test(|root| {
		let path = path_cstr(root.join("lseek.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let lseek_rc = unsafe { libc::lseek(file_fd, 1024, libc::SEEK_HOLE) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(lseek_rc, 4096);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"LseekRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    whence: SEEK_HOLE,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
