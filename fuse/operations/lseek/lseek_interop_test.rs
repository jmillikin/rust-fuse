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
		if request.name() != "lseek.txt" {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::Attributes::new(fuse::NodeId::new(2).unwrap());
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

	fn lseek(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::LseekRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		let mut reply = fuse::kernel::fuse_lseek_out::new();
		reply.offset = 4096;
		send_reply.ok(&reply).unwrap();
	}
}

fn lseek_test(
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
fn lseek_set() {
	let requests = lseek_test(|root| {
		let path = path_cstr(root.join("lseek.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let lseek_rc = unsafe { libc::lseek(file_fd, 1024, libc::SEEK_SET) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(lseek_rc, 1024);
	});
	assert_eq!(requests.len(), 0);
}

#[test]
fn lseek_data() {
	let requests = lseek_test(|root| {
		let path = path_cstr(root.join("lseek.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let lseek_rc = unsafe { libc::lseek(file_fd, 1024, libc::SEEK_DATA) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(lseek_rc, 4096);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"LseekRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    whence: SEEK_DATA,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn lseek_hole() {
	let requests = lseek_test(|root| {
		let path = path_cstr(root.join("lseek.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);
		let lseek_rc = unsafe { libc::lseek(file_fd, 1024, libc::SEEK_HOLE) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(lseek_rc, 4096);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"LseekRequest {
    node_id: 2,
    handle: 12345,
    offset: 1024,
    whence: SEEK_HOLE,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
