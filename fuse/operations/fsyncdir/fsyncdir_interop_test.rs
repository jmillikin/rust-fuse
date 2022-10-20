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
		if request.name() != "fsyncdir.d" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = node::Attributes::new(node::Id::new(2).unwrap());
		attr.set_mode(node::Mode::S_IFDIR | 0o755);
		attr.set_link_count(2);

		let mut entry = node::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn opendir(
		&self,
		call: fuse_rpc::Call<S>,
		_request: &OpendirRequest,
	) -> fuse_rpc::SendResult<OpendirResponse, S::Error> {
		let mut resp = OpendirResponse::new();
		resp.set_handle(12345);
		call.respond_ok(&resp)
	}

	fn fsyncdir(
		&self,
		call: fuse_rpc::Call<S>,
		request: &FsyncdirRequest,
	) -> fuse_rpc::SendResult<FsyncdirResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let resp = FsyncdirResponse::new();
		call.respond_ok(&resp)
	}
}

fn fsyncdir_test(
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
fn fsyncdir_fsync() {
	let requests = fsyncdir_test(|root| {
		let path = path_cstr(root.join("fsyncdir.d"));

		let dir_fd = unsafe { libc::open(path.as_ptr(), 0) };
		assert_ne!(dir_fd, -1);
		let fsync_rc = unsafe { libc::fsync(dir_fd) };
		unsafe {
			libc::close(dir_fd)
		};
		assert_eq!(fsync_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FsyncdirRequest {
    node_id: 2,
    handle: 12345,
    flags: FsyncdirRequestFlags {},
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn fsyncdir_fdatasync() {
	let requests = fsyncdir_test(|root| {
		let path = path_cstr(root.join("fsyncdir.d"));

		let dir_fd = unsafe { libc::open(path.as_ptr(), 0) };
		assert_ne!(dir_fd, -1);
		let fsync_rc = unsafe { libc::fdatasync(dir_fd) };
		unsafe {
			libc::close(dir_fd)
		};
		assert_eq!(fsync_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FsyncdirRequest {
    node_id: 2,
    handle: 12345,
    flags: FsyncdirRequestFlags {
        FDATASYNC,
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
