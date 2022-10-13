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
	errno,
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
		if request.name() != "symlink.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = node::Attributes::new(node::Id::new(2).unwrap());
		attr.set_mode(node::Mode::S_IFLNK | 0o644);
		attr.set_link_count(1);

		let mut entry = node::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = fuse::LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn readlink(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::ReadlinkRequest,
	) -> fuse_rpc::FuseResult<fuse::ReadlinkResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let name = node::Name::new("target.txt").unwrap();
		let resp = fuse::ReadlinkResponse::from_name(name);
		call.respond_ok(&resp)
	}
}

fn readlink_test(
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
fn readlink() {
	let requests = readlink_test(|root| {
		let path = path_cstr(root.join("symlink.txt"));

		let mut value = [0u8; 15];
		let rc = unsafe {
			libc::readlink(
				path.as_ptr(),
				value.as_mut_ptr() as *mut i8,
				value.len(),
			)
		};

		assert_eq!(rc, 10);
		assert_eq!(&value, b"target.txt\0\0\0\0\0")
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"ReadlinkRequest {
    node_id: 2,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn readlink_err_enoent() {
	let requests = readlink_test(|root| {
		let path = path_cstr(root.join("noexist.txt"));

		let mut value = [0i8; 15];
		let rc = unsafe {
			libc::readlink(path.as_ptr(), value.as_mut_ptr(), value.len())
		};
		assert_eq!(rc, -1);
		assert_eq!(errno(), libc::ENOENT);
	});
	assert_eq!(requests.len(), 0);
}
