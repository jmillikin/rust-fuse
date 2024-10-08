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

use fuse::server::fuse_rpc;
use fuse::server::prelude::*;

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	libc_errno,
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
		if request.name() != "exists.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = fuse::Attributes::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn symlink(
		&self,
		call: fuse_rpc::Call<S>,
		request: &SymlinkRequest,
	) -> fuse_rpc::SendResult<SymlinkResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut attr = fuse::Attributes::new(fuse::NodeId::new(3).unwrap());
		attr.set_mode(fuse::FileMode::S_IFLNK | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = SymlinkResponse::new(entry);
		call.respond_ok(&resp)
	}
}

fn symlink_test(
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
fn symlink() {
	let requests = symlink_test(|root| {
		let path = path_cstr(root.join("symlink.txt"));
		let target = c"symlink_target.txt";

		let rc = unsafe { libc::symlink(target.as_ptr(), path.as_ptr()) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SymlinkRequest {
    parent_id: 1,
    name: "symlink_target.txt",
    content: "symlink.txt",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn symlink_err_eexist() {
	let requests = symlink_test(|root| {
		let path = path_cstr(root.join("exists.txt"));
		let target = c"symlink_target.txt";

		let rc = unsafe { libc::symlink(target.as_ptr(), path.as_ptr()) };
		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::EEXIST);
	});
	assert_eq!(requests.len(), 0);
}
