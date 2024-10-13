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
		if request.name() != "fsync.txt" {
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

	fn fsync(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::FsyncRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();
		send_reply.ok_empty().unwrap();
	}
}

fn fsync_test(
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
fn fsync() {
	let requests = fsync_test(|root| {
		let path = path_cstr(root.join("fsync.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), 0) };
		assert_ne!(file_fd, -1);
		let fsync_rc = unsafe { libc::fsync(file_fd) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(fsync_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FsyncRequest {
    node_id: 2,
    handle: 12345,
    flags: FsyncRequestFlags {},
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn fsync_fdatasync() {
	let requests = fsync_test(|root| {
		let path = path_cstr(root.join("fsync.txt"));

		let file_fd = unsafe { libc::open(path.as_ptr(), 0) };
		assert_ne!(file_fd, -1);
		let fsync_rc = unsafe { libc::fdatasync(file_fd) };
		unsafe {
			libc::close(file_fd)
		};
		assert_eq!(fsync_rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"FsyncRequest {
    node_id: 2,
    handle: 12345,
    flags: FsyncRequestFlags {
        FDATASYNC,
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
