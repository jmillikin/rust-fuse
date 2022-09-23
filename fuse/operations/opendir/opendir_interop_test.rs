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
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if request.parent_id() != fuse::ROOT_ID {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != fuse::NodeName::from_bytes(b"opendir.d").unwrap() {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Directory | 0o755);
		attr.set_nlink(2);

		call.respond_ok(&resp)
	}

	fn opendir(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::OpendirRequest,
	) -> fuse_rpc::FuseResult<fuse::OpendirResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = fuse::OpendirResponse::new();
		resp.set_handle(12345);
		call.respond_ok(&resp)
	}
}

fn opendir_test(
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
fn opendir() {
	let requests = opendir_test(|root| {
		let path = path_cstr(root.join("opendir.d"));

		let dir_p = unsafe { libc::opendir(path.as_ptr()) };
		assert!(!dir_p.is_null());
		unsafe {
			libc::closedir(dir_p)
		};
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let expect = r#"OpendirRequest {
    node_id: 2,
    flags: 0x00018000,
}"#;
	#[cfg(target_os = "freebsd")]
	let expect = r#"OpendirRequest {
    node_id: 2,
    flags: 0x00000000,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn opendir_open() {
	let requests = opendir_test(|root| {
		let path = path_cstr(root.join("opendir.d"));

		let dir_fd = unsafe { libc::open(path.as_ptr(), 0) };
		assert_ne!(dir_fd, -1);
		unsafe {
			libc::close(dir_fd)
		};
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let expect = r#"OpendirRequest {
    node_id: 2,
    flags: 0x00008000,
}"#;
	#[cfg(target_os = "freebsd")]
	let expect = r#"OpendirRequest {
    node_id: 2,
    flags: 0x00000000,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
