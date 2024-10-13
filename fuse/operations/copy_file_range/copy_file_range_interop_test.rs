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

use linux_syscall::ResultSize;

use fuse::server;
use fuse::server::FuseRequest;

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	path_cstr,
	OsError,
};

struct TestFS {
	requests: mpsc::Sender<String>,
}

struct TestHandlers<'a, S> {
	fs: &'a TestFS,
	conn: &'a server::FuseConnection<S>,
}

impl interop_testutil::TestFS for TestFS {
	fn dispatch_request(
		&self,
		conn: &server::FuseConnection<interop_testutil::DevFuse>,
		request: FuseRequest<'_>,
	) {
		use fuse::server::FuseHandlers;
		(TestHandlers{fs: self, conn}).dispatch(request);
	}
}

impl<'a, S> server::FuseHandlers for TestHandlers<'a, S>
where
	S: server::FuseSocket,
	S::Error: core::fmt::Debug,
{
	fn unimplemented(&self, request: FuseRequest<'_>) {
		self.conn.reply(request.id()).err(OsError::UNIMPLEMENTED).unwrap();
	}

	fn copy_file_range(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::CopyFileRangeRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		let mut reply = fuse::kernel::fuse_write_out::new();
		reply.size = 500;
		send_reply.ok(&reply).unwrap();
	}

	fn lookup(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::LookupRequest::try_from(request).unwrap();

		if !request.parent_id().is_root() {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let node_id;
		if request.name() == "file_src.txt" {
			node_id = fuse::NodeId::new(2).unwrap();
		} else if request.name() == "file_dst.txt" {
			node_id = fuse::NodeId::new(3).unwrap();
		} else {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::NodeAttr::new(node_id);
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);
		attr.set_size(1_000_000);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn open(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let node_id = request.header().raw().nodeid;
		let request = server::OpenRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		let mut reply = fuse::kernel::fuse_open_out::new();
		if node_id == 2 {
			reply.fh = 10;
		} else if node_id == 3 {
			reply.fh = 20;
		} else {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}
		send_reply.ok(&reply).unwrap();
	}

	fn release(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReleaseRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();
		send_reply.ok_empty().unwrap();
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
