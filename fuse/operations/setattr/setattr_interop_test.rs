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

use fuse::NodeId;
use fuse::server::fuse_rpc;

use interop_testutil::{diff_str, fuse_interop_test, path_cstr, ErrorCode};

struct TestFS {
	opts: TestOptions,
	requests: mpsc::Sender<String>,
}

struct TestOptions {
	stub_lock_owner: bool,
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

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		if request.name() == "file.txt" {
			node.set_id(fuse::NodeId::new(2).unwrap());

			let attr = node.attr_mut();
			attr.set_mode(fuse::FileType::Regular | 0o644);
			attr.set_nlink(1);

			return call.respond_ok(&resp);
		}

		call.respond_err(ErrorCode::ENOENT)
	}

	fn getattr(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::GetattrRequest,
	) -> fuse_rpc::FuseResult<fuse::GetattrResponse, S::Error> {
		let mut resp = fuse::GetattrResponse::new();
		let attr = resp.attr_mut();

		if request.node_id() == fuse::ROOT_ID {
			attr.set_mode(fuse::FileType::Directory | 0o755);
			attr.set_nlink(2);
			return call.respond_ok(&resp);
		}

		if request.node_id() == fuse::NodeId::new(2).unwrap() {
			attr.set_mode(fuse::FileType::Regular | 0o644);
			attr.set_nlink(1);
			return call.respond_ok(&resp);
		}

		call.respond_err(ErrorCode::ENOENT)
	}

	fn open(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::OpenRequest,
	) -> fuse_rpc::FuseResult<fuse::OpenResponse, S::Error> {
		let mut resp = fuse::OpenResponse::new();
		if request.node_id() == fuse::NodeId::new(2).unwrap() {
			resp.set_handle(1002);
			return call.respond_ok(&resp);
		}
		call.respond_err(ErrorCode::ENOENT)
	}

	fn setattr(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::SetattrRequest,
	) -> fuse_rpc::FuseResult<fuse::SetattrResponse, S::Error> {
		println!("{:#?}", request);

		let mut request_str = format!("{:#?}", request);

		if self.opts.stub_lock_owner {
			// stub out the lock owner, which is non-deterministic.
			let repl_start = request_str.find("lock_owner:").unwrap();
			let repl_end =
				repl_start + request_str[repl_start..].find(",").unwrap();
			request_str.replace_range(
				repl_start..=repl_end,
				"lock_owner: FAKE_LOCK_OWNER,",
			);
		}

		self.requests.send(request_str).unwrap();

		let mut resp = fuse::SetattrResponse::new();
		let attr = resp.attr_mut();
		attr.set_node_id(NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		call.respond_ok(&resp)
	}
}

fn setattr_test(
	opts: Option<TestOptions>,
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		opts: opts.unwrap_or(TestOptions {
			stub_lock_owner: false,
		}),
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

#[test]
fn setattr_chown() {
	let requests = setattr_test(None, |root| {
		let path = path_cstr(root.join("file.txt"));

		let rc = unsafe { libc::chown(path.as_ptr(), 123, 456) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetattrRequest {
    node_id: 2,
    handle: None,
    size: None,
    lock_owner: None,
    atime: None,
    atime_now: false,
    mtime: None,
    mtime_now: false,
    ctime: None,
    mode: None,
    user_id: Some(123),
    group_id: Some(456),
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn setattr_chmod() {
	let requests = setattr_test(None, |root| {
		let path = path_cstr(root.join("file.txt"));

		let rc = unsafe { libc::chmod(path.as_ptr(), 0o755) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let expect_mode = "0o100755";

	#[cfg(target_os = "freebsd")]
	let expect_mode = "0o755";

	let expect = format!(
		r#"SetattrRequest {{
    node_id: 2,
    handle: None,
    size: None,
    lock_owner: None,
    atime: None,
    atime_now: false,
    mtime: None,
    mtime_now: false,
    ctime: None,
    mode: Some({mode}),
    user_id: None,
    group_id: None,
}}"#,
		mode = expect_mode,
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn setattr_utimens() {
	let requests = setattr_test(None, |root| {
		let path = path_cstr(root.join("file.txt"));

		let times = [
			// atime
			libc::timeval {
				tv_sec: 1400000000,
				tv_usec: 1234,
			},
			// mtime
			libc::timeval {
				tv_sec: 1500000000,
				tv_usec: 5678,
			},
		];

		let rc = unsafe { libc::utimes(path.as_ptr(), (&times).as_ptr()) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetattrRequest {
    node_id: 2,
    handle: None,
    size: None,
    lock_owner: None,
    atime: Some(1400000000.001234s),
    atime_now: false,
    mtime: Some(1500000000.005678s),
    mtime_now: false,
    ctime: None,
    mode: None,
    user_id: None,
    group_id: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn setattr_truncate() {
	let opts = TestOptions {
		stub_lock_owner: true,
	};
	let requests = setattr_test(Some(opts), |root| {
		let path = path_cstr(root.join("file.txt"));

		let rc = unsafe { libc::truncate(path.as_ptr(), 12345) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetattrRequest {
    node_id: 2,
    handle: None,
    size: Some(12345),
    lock_owner: FAKE_LOCK_OWNER,
    atime: None,
    atime_now: false,
    mtime: None,
    mtime_now: false,
    ctime: None,
    mode: None,
    user_id: None,
    group_id: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn setattr_ftruncate() {
	let opts = TestOptions {
		stub_lock_owner: true,
	};
	let requests = setattr_test(Some(opts), |root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let rc = unsafe { libc::ftruncate(fd, 12345) };
		assert_eq!(rc, 0);

		unsafe { libc::close(fd) };
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetattrRequest {
    node_id: 2,
    handle: Some(1002),
    size: Some(12345),
    lock_owner: FAKE_LOCK_OWNER,
    atime: None,
    atime_now: false,
    mtime: None,
    mtime_now: false,
    ctime: None,
    mode: None,
    user_id: None,
    group_id: None,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
