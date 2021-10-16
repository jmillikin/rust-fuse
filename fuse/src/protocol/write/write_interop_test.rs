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

use interop_testutil::{cuse_interop_test, diff_str, path_cstr};

struct TestCharDev {
	requests: mpsc::Sender<String>,
}

impl fuse::CuseHandlers for TestCharDev {
	fn cuse_init(
		&mut self,
		_request: &fuse::CuseInitRequest,
	) -> fuse::CuseInitResponse {
		let mut reply = fuse::CuseInitResponse::new();
		reply.set_dev_major(100);
		reply.set_dev_minor(101);
		reply
	}

	fn open(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpenRequest,
		respond: impl for<'a> fuse::Respond<fuse::OpenResponse<'a>>,
	) {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(12345);
		respond.ok(&resp);
	}

	fn release(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReleaseRequest,
		respond: impl for<'a> fuse::Respond<fuse::ReleaseResponse<'a>>,
	) {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let resp = fuse::ReleaseResponse::new();
		respond.ok(&resp);
	}

	fn write(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::WriteRequest,
		respond: impl for<'a> fuse::Respond<fuse::WriteResponse<'a>>,
	) {
		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let repl_start = request_str.find("lock_owner:").unwrap();
		let repl_end =
			repl_start + request_str[repl_start..].find(",").unwrap();
		request_str.replace_range(
			repl_start..=repl_end,
			"lock_owner: FAKE_LOCK_OWNER,",
		);

		self.requests.send(request_str).unwrap();

		let mut resp = fuse::WriteResponse::new();
		resp.set_size(request.value().len() as u32);
		respond.ok(&resp);
	}
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
fn cuse_write() {
	let requests = cuse_write_test(|dev_path| {
		let path = path_cstr(dev_path.to_owned());

		let mknod_rc = unsafe {
			let dev_t = libc::makedev(100, 101);
			libc::mknod(path.as_ptr(), libc::S_IFCHR | 0o777, dev_t)
		};
		assert_eq!(mknod_rc, 0);

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
    flags: 0x00008002,
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
        write_cache: false,
        0x00000002: true,
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
