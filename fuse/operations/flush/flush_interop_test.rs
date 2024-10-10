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

use fuse::server::fuse_rpc;
use fuse::server::prelude::*;

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

impl interop_testutil::TestFS for TestFS {}

impl<S: FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &LookupRequest,
	) -> fuse_rpc::SendResult<LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(OsError::NOT_FOUND);
		}

		let node_id;
		if request.name() == "flush.txt" {
			node_id = fuse::NodeId::new(2).unwrap();
		} else if request.name() == "flush_err.txt" {
			node_id = fuse::NodeId::new(3).unwrap();
		} else {
			return call.respond_err(OsError::NOT_FOUND);
		}

		let mut attr = fuse::Attributes::new(node_id);
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		request: &OpenRequest,
	) -> fuse_rpc::SendResult<OpenResponse, S::Error> {
		let mut resp = OpenResponse::new();
		if request.node_id() == fuse::NodeId::new(2).unwrap() {
			resp.set_handle(1002);
		} else {
			resp.set_handle(1003);
		}
		call.respond_ok(&resp)
	}

	fn flush(
		&self,
		call: fuse_rpc::Call<S>,
		request: &FlushRequest,
	) -> fuse_rpc::SendResult<FlushResponse, S::Error> {
		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let repl_start = request_str.find("lock_owner:").unwrap();
		let repl_end =
			repl_start + request_str[repl_start..].find(",").unwrap();
		request_str.replace_range(
			repl_start..=repl_end,
			"lock_owner: FAKE_LOCK_OWNER,",
		);

		self.requests.send(request_str).unwrap();

		if request.handle() == 1002 {
			let resp = FlushResponse::new();
			call.respond_ok(&resp)
		} else {
			call.respond_err(OsError::from(errno::E2BIG))
		}
	}
}

fn flush_test(
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
fn flush() {
	let requests = flush_test(|root| {
		let path = path_cstr(root.join("flush.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let rc = unsafe { libc::close(file_fd) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FlushRequest {
    node_id: 2,
    handle: 1002,
    lock_owner: FAKE_LOCK_OWNER,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn flush_err() {
	let requests = flush_test(|root| {
		let path = path_cstr(root.join("flush_err.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let rc = unsafe { libc::close(file_fd) };
		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::E2BIG);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FlushRequest {
    node_id: 3,
    handle: 1003,
    lock_owner: FAKE_LOCK_OWNER,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
