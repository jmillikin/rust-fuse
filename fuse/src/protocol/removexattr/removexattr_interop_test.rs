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
		if request.name() != fuse::NodeName::from_bytes(b"xattrs.txt").unwrap()
		{
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		call.respond_ok(&resp)
	}

	fn removexattr(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::RemovexattrRequest,
	) -> fuse_rpc::FuseResult<fuse::RemovexattrResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();
		let resp = fuse::RemovexattrResponse::new();
		call.respond_ok(&resp)
	}
}

fn removexattr_test(
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
fn removexattr() {
	let requests = removexattr_test(|root| {
		let path = path_cstr(root.join("xattrs.txt"));

		#[cfg(target_os = "linux")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("user.xattr_name").unwrap();
			libc::removexattr(
				path.as_ptr(),
				xattr_name.as_ptr(),
			)
		};

		#[cfg(target_os = "freebsd")]
		let rc = unsafe {
			let xattr_name = ffi::CString::new("xattr_name").unwrap();
			libc::extattr_delete_file(
				path.as_ptr(),
				libc::EXTATTR_NAMESPACE_USER,
				xattr_name.as_ptr(),
			)
		};

		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"RemovexattrRequest {
    node_id: 2,
    name: "user.xattr_name",
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
