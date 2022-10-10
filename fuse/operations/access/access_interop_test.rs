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
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "access.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(node::Id::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(node::Mode::S_IFREG | 0o755);
		attr.set_nlink(2);

		call.respond_ok(&resp)
	}

	fn access(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::AccessRequest,
	) -> fuse_rpc::FuseResult<fuse::AccessResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let resp = fuse::AccessResponse::new();
		call.respond_ok(&resp)
	}

	fn getattr(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::GetattrRequest,
	) -> fuse_rpc::FuseResult<fuse::GetattrResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = fuse::GetattrResponse::new();
		let attr = resp.attr_mut();
		attr.set_mode(node::Mode::S_IFREG | 0o755);
		attr.set_nlink(2);

		call.respond_ok(&resp)
	}
}

fn access_test(
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
fn access() {
	let requests = access_test(|root| {
		let path = path_cstr(root.join("access.txt"));

		let rc = unsafe { libc::access(path.as_ptr(), libc::F_OK) };
		assert_eq!(rc, 0);
	});

	#[cfg(target_os = "linux")]
	{
		assert_eq!(requests.len(), 1);
		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 0,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}
	}

	#[cfg(target_os = "freebsd")]
	{
		assert_eq!(requests.len(), 1);
		let expect = r#"AccessRequest {
    node_id: 1,
    mask: 1,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
fn access_read() {
	let requests = access_test(|root| {
		let path = path_cstr(root.join("access.txt"));

		let rc = unsafe { libc::access(path.as_ptr(), libc::R_OK) };
		assert_eq!(rc, 0);
	});

	#[cfg(target_os = "linux")]
	{
		assert_eq!(requests.len(), 1);

		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 4,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}
	}

	#[cfg(target_os = "freebsd")]
	{
		assert_eq!(requests.len(), 2);

		let expect = r#"AccessRequest {
    node_id: 1,
    mask: 1,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 4,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
fn access_write() {
	let requests = access_test(|root| {
		let path = path_cstr(root.join("access.txt"));

		let rc = unsafe { libc::access(path.as_ptr(), libc::W_OK) };
		assert_eq!(rc, 0);
	});

	#[cfg(target_os = "linux")]
	{
		assert_eq!(requests.len(), 1);

		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 2,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}
	}

	#[cfg(target_os = "freebsd")]
	{
		assert_eq!(requests.len(), 2);

		let expect = r#"AccessRequest {
    node_id: 1,
    mask: 1,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 2,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
fn access_exec() {
	let requests = access_test(|root| {
		let path = path_cstr(root.join("access.txt"));

		let rc = unsafe { libc::access(path.as_ptr(), libc::X_OK) };
		assert_eq!(rc, 0);
	});

	#[cfg(target_os = "linux")]
	{
		assert_eq!(requests.len(), 2);

		let expect = r#"GetattrRequest {
    node_id: 2,
    handle: None,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 1,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}

	#[cfg(target_os = "freebsd")]
	{
		assert_eq!(requests.len(), 2);

		let expect = r#"AccessRequest {
    node_id: 1,
    mask: 1,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"AccessRequest {
    node_id: 2,
    mask: 1,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}
