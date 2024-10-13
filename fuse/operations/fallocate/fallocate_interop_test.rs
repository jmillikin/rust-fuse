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

	fn lookup(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::LookupRequest::try_from(request).unwrap();

		if !request.parent_id().is_root() {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}
		if request.name() != "fallocate.txt" {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn open(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 12345;
		send_reply.ok(&reply).unwrap();
	}

	fn fallocate(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::FallocateRequest::try_from(request).unwrap();

		self.fs.requests.send(format!("{:#?}", request)).unwrap();
		send_reply.ok_empty().unwrap();
	}
}

fn fallocate_test(
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
fn fallocate() {
	let requests = fallocate_test(|root| {
		let path = path_cstr(root.join("fallocate.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let fallocate_rc = unsafe { libc::fallocate(file_fd, 0, 1024, 4096) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(fallocate_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FallocateRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    length: 4096,
    fallocate_flags: 0x00000000,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn posix_fallocate() {
	let requests = fallocate_test(|root| {
		let path = path_cstr(root.join("fallocate.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let fallocate_rc = unsafe { libc::posix_fallocate(file_fd, 1024, 4096) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(fallocate_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FallocateRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    length: 4096,
    fallocate_flags: 0x00000000,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn fallocate_keep_size() {
	let requests = fallocate_test(|root| {
		let path = path_cstr(root.join("fallocate.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let fallocate_rc = unsafe {
			libc::fallocate(file_fd, libc::FALLOC_FL_KEEP_SIZE, 1024, 4096)
		};
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(fallocate_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FallocateRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    length: 4096,
    fallocate_flags: 0x00000001,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn fallocate_punch_hole() {
	let requests = fallocate_test(|root| {
		let path = path_cstr(root.join("fallocate.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let fallocate_rc = unsafe {
			libc::fallocate(
				file_fd,
				libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
				1024,
				4096,
			)
		};
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(fallocate_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FallocateRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    length: 4096,
    fallocate_flags: 0x00000003,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
