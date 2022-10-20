// Copyright 2022 John Millikin and the rust-fuse contributors.
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
use linux_syscall::ResultSize;

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
	fn copy_file_range(
		&self,
		call: fuse_rpc::Call<S>,
		request: &CopyFileRangeRequest,
	) -> fuse_rpc::SendResult<CopyFileRangeResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = CopyFileRangeResponse::new();
		resp.set_size(500);
		call.respond_ok(&resp)
	}

	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &LookupRequest,
	) -> fuse_rpc::SendResult<LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let node_id;
		if request.name() == "file_src.txt" {
			node_id = node::Id::new(2).unwrap();
		} else if request.name() == "file_dst.txt" {
			node_id = node::Id::new(3).unwrap();
		} else {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = node::Attributes::new(node_id);
		attr.set_mode(node::Mode::S_IFREG | 0o644);
		attr.set_link_count(1);
		attr.set_size(1_000_000);

		let mut entry = node::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		request: &OpenRequest,
	) -> fuse_rpc::SendResult<OpenResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = OpenResponse::new();
		if request.node_id().get() == 2 {
			resp.set_handle(10);
		} else if request.node_id().get() == 3 {
			resp.set_handle(20);
		} else {
			return call.respond_err(ErrorCode::ENOENT);
		}
		call.respond_ok(&resp)
	}

	fn release(
		&self,
		call: fuse_rpc::Call<S>,
		request: &ReleaseRequest,
	) -> fuse_rpc::SendResult<ReleaseResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let resp = ReleaseResponse::new();
		call.respond_ok(&resp)
	}
}

fn copy_file_range_test(
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
fn copy_file_range() {
	let requests = copy_file_range_test(|root| {
		let path_src = path_cstr(root.join("file_src.txt"));
		let path_dst = path_cstr(root.join("file_dst.txt"));

		let file_src_fd = unsafe {
			libc::open(path_src.as_ptr(), libc::O_RDONLY)
		};
		assert_ne!(file_src_fd, -1);

		let file_dst_fd = unsafe {
			libc::open(path_dst.as_ptr(), libc::O_WRONLY)
		};
		assert_ne!(file_dst_fd, -1);

		let copy_len: usize = 1000;
		let flags: libc::c_uint = 0;
		let mut input_offset: usize = 1234;
		let mut output_offset: usize = 5678;
		let copied_len = unsafe {
			linux_syscall::syscall!(
				linux_syscall::SYS_copy_file_range,
				file_src_fd,
				&mut input_offset,
				file_dst_fd,
				&mut output_offset,
				copy_len,
				flags,
			).try_usize().unwrap()
		};

		assert_eq!(copied_len, 500);
		assert_eq!(input_offset, 1234 + 500);
		assert_eq!(output_offset, 5678 + 500);

		unsafe {
			libc::close(file_src_fd);
			libc::close(file_dst_fd);
		}
	});
	assert_eq!(requests.len(), 3);

	let expect = format!(
		r#"OpenRequest {{
    node_id: 2,
    flags: OpenRequestFlags {{}},
    open_flags: 0x00008000,
}}"#,
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}

	let expect = format!(
		r#"OpenRequest {{
    node_id: 3,
    flags: OpenRequestFlags {{}},
    open_flags: 0x00008001,
}}"#,
	);
	if let Some(diff) = diff_str(&expect, &requests[1]) {
		println!("{}", diff);
		assert!(false);
	}

	let expect = format!(
		r#"CopyFileRangeRequest {{
    input_node_id: 2,
    input_handle: 10,
    input_offset: 1234,
    output_node_id: 3,
    output_handle: 20,
    output_offset: 5678,
    len: 1000,
    flags: CopyFileRangeRequestFlags {{}},
}}"#,
	);
	if let Some(diff) = diff_str(&expect, &requests[2]) {
		println!("{}", diff);
		assert!(false);
	}
}
