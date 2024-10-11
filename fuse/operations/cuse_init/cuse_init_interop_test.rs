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
use fuse::server::CuseRequest;

use interop_testutil::{
	cuse_interop_test,
	diff_str,
	path_cstr,
	OsError,
};

struct TestCharDev {
	requests: mpsc::Sender<String>,
}

struct TestHandlers<'a, S> {
	dev: &'a TestCharDev,
	conn: &'a server::CuseConnection<S>,
}

impl interop_testutil::TestDev for TestCharDev {
	fn dispatch_request(
		&self,
		conn: &server::CuseConnection<interop_testutil::DevCuse>,
		request: CuseRequest<'_>,
	) {
		use fuse::server::CuseHandlers;
		(TestHandlers{dev: self, conn}).dispatch(request);
	}
}

impl<'a, S> server::CuseHandlers for TestHandlers<'a, S>
where
	S: server::CuseSocket,
	S::Error: core::fmt::Debug,
{
	fn unimplemented(&self, request: CuseRequest<'_>) {
		self.conn.reply(request.id()).err(OsError::UNIMPLEMENTED).unwrap();
	}

	fn open(&self, request: CuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::OpenRequest::try_from(request).unwrap();
		self.dev.requests.send(format!("{:#?}", request)).unwrap();

		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 12345;
		send_reply.ok(&reply).unwrap();
	}

	fn read(&self, request: CuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReadRequest::try_from(request).unwrap();
		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let repl_start = request_str.find("lock_owner:").unwrap();
		let repl_end =
			repl_start + request_str[repl_start..].find(",").unwrap();
		request_str.replace_range(
			repl_start..=repl_end,
			"lock_owner: FAKE_LOCK_OWNER,",
		);

		self.dev.requests.send(request_str).unwrap();
		send_reply.ok_buf(b"file_content").unwrap();
	}

	fn release(&self, request: CuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReleaseRequest::try_from(request).unwrap();
		self.dev.requests.send(format!("{:#?}", request)).unwrap();
		send_reply.ok_empty().unwrap();
	}

	fn write(&self, request: CuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::WriteRequest::try_from(request).unwrap();
		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let repl_start = request_str.find("lock_owner:").unwrap();
		let repl_end =
			repl_start + request_str[repl_start..].find(",").unwrap();
		request_str.replace_range(
			repl_start..=repl_end,
			"lock_owner: FAKE_LOCK_OWNER,",
		);

		self.dev.requests.send(request_str).unwrap();

		let mut reply = fuse::kernel::fuse_write_out::new();
		reply.size = request.value().len() as u32;
		send_reply.ok(&reply).unwrap();
	}
}

fn cuse_read_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let chardev = TestCharDev {
		requests: request_send,
	};
	cuse_interop_test(chardev, test_fn);
	request_recv.iter().collect()
}

fn cuse_write_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let chardev = TestCharDev {
		requests: request_send,
	};
	cuse_interop_test(chardev, test_fn);
	request_recv.iter().collect()
}

#[test]
fn cuse_read() {
	let requests = cuse_read_test(|dev_path| {
		let path = path_cstr(dev_path.to_owned());

		let mut value = [0u8; 15];

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);

		let read_rc = unsafe {
			libc::read(
				file_fd,
				value.as_mut_ptr() as *mut libc::c_void,
				value.len(),
			)
		};
		unsafe { libc::close(file_fd) };

		assert!(read_rc > 0);
		assert_eq!(&value, b"file_content\0\0\0")
	});

	{
		assert_eq!(requests.len(), 3);

		let expect = r#"OpenRequest {
    node_id: 1,
    flags: OpenRequestFlags {},
    open_flags: 0x00008002,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"ReadRequest {
    node_id: 1,
    size: 15,
    offset: 0,
    handle: 12345,
    lock_owner: FAKE_LOCK_OWNER,
    open_flags: 0x00008002,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"ReleaseRequest {
    node_id: 1,
    handle: 12345,
    lock_owner: None,
    open_flags: 0x00008002,
}"#;
		if let Some(diff) = diff_str(expect, &requests[2]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
fn cuse_write() {
	let requests = cuse_write_test(|dev_path| {
		let path = path_cstr(dev_path.to_owned());

		let file_fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(file_fd, -1);

		let value = b"new_file_content";
		let write_rc = unsafe {
			libc::write(
				file_fd,
				value.as_ptr() as *const libc::c_void,
				value.len(),
			)
		};
		unsafe { libc::close(file_fd) };

		assert_eq!(write_rc, value.len() as isize);
	});

	{
		assert_eq!(requests.len(), 3);

		let expect = r#"OpenRequest {
    node_id: 1,
    flags: OpenRequestFlags {},
    open_flags: 0x00008002,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"WriteRequest {
    node_id: 1,
    offset: 0,
    handle: 12345,
    value: "new_file_content",
    flags: WriteRequestFlags {
        WRITE_LOCKOWNER,
    },
    lock_owner: FAKE_LOCK_OWNER,
    open_flags: 0x00008002,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"ReleaseRequest {
    node_id: 1,
    handle: 12345,
    lock_owner: None,
    open_flags: 0x00008002,
}"#;
		if let Some(diff) = diff_str(expect, &requests[2]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}
