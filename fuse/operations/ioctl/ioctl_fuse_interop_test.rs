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

use std::mem::size_of;
use std::panic;
use std::sync::mpsc;

use fuse::node;
use fuse::server::fuse_rpc;

use interop_testutil::{diff_str, path_cstr, ErrorCode};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl interop_testutil::TestFS for TestFS {}

impl<S: fuse_rpc::FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "file.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = node::Attributes::new(node::Id::new(2).unwrap());
		attr.set_mode(node::Mode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = node::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = fuse::LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn getattr(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::GetattrRequest,
	) -> fuse_rpc::FuseResult<fuse::GetattrResponse, S::Error> {
		println!("{:#?}", request);

		let mut attr = node::Attributes::new(request.node_id());

		if request.node_id().is_root() {
			attr.set_mode(node::Mode::S_IFDIR | 0o755);
			attr.set_link_count(2);
			let resp = fuse::GetattrResponse::new(attr);
			return call.respond_ok(&resp);
		}

		if request.node_id() == node::Id::new(2).unwrap() {
			attr.set_mode(node::Mode::S_IFREG | 0o644);
			attr.set_link_count(1);
			let resp = fuse::GetattrResponse::new(attr);
			return call.respond_ok(&resp);
		}

		call.respond_err(ErrorCode::ENOENT)
	}

	fn ioctl(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::IoctlRequest,
	) -> fuse_rpc::FuseResult<fuse::IoctlResponse, S::Error> {
		println!("{:#?}", request);

		let mut request_str = format!("{:#?}", request);

		// stub out the ioctl arg, which is non-deterministic.
		let repl_start = request_str.find("arg:").unwrap();
		let repl_end =
			repl_start + request_str[repl_start..].find(",").unwrap();
		request_str.replace_range(
			repl_start..=repl_end,
			"arg: FAKE_ARG,",
		);

		self.requests.send(request_str).unwrap();

		if request.command().get() == libc::TCGETS2 as u32 {
			let response_value = libc::termios2 {
				c_iflag: 12,
				c_oflag: 34,
				c_cflag: 56,
				c_lflag: 78,
				c_line: 0,
				c_cc: [0; 19],
				c_ispeed: 0,
				c_ospeed: 0,
			};
			let response_buf: &[u8] = unsafe {
				core::slice::from_raw_parts(
					(&response_value as *const libc::termios2) as *const u8,
					size_of::<libc::termios2>(),
				)
			};
			let resp = fuse::IoctlResponse::new(response_buf);
			return call.respond_ok(&resp);
		}

		if request.command().get() == libc::TCSETS2 as u32 {
			let resp = fuse::IoctlResponse::new(b"");
			return call.respond_ok(&resp);
		}

		call.respond_err(ErrorCode::EOPNOTSUPP)
	}

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::OpenRequest,
	) -> fuse_rpc::FuseResult<fuse::OpenResponse, S::Error> {
		println!("{:#?}", request);
		let mut resp = fuse::OpenResponse::new();
		if request.node_id() == node::Id::new(2).unwrap() {
			resp.set_handle(1002);
			return call.respond_ok(&resp);
		}
		call.respond_err(ErrorCode::ENOENT)
	}
}

fn fuse_ioctl_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	interop_testutil::fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

fn indented_input(input: &[u8]) -> String {
	format!("{:#?}", input)
		.replace("[\n", "[\n    ")
		.replace(",\n ", ",\n     ")
		.replace("]", "    ]")
}

#[test]
fn fuse_ioctl_tcgets2() {
	let requests = fuse_ioctl_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut termios2_buf = libc::termios2 {
			c_iflag: 0,
			c_oflag: 0,
			c_cflag: 0,
			c_lflag: 0,
			c_line: 0,
			c_cc: [0; 19],
			c_ispeed: 0,
			c_ospeed: 0,
		};
		let rc = unsafe {
			libc::ioctl(
				fd,
				libc::TCGETS2,
				&mut termios2_buf as *mut libc::termios2,
			)
		};
		assert_eq!(rc, 0);
		assert_eq!(termios2_buf.c_iflag, 12);
		assert_eq!(termios2_buf.c_oflag, 34);
		assert_eq!(termios2_buf.c_cflag, 56);
		assert_eq!(termios2_buf.c_lflag, 78);

		unsafe { libc::close(fd) };

	});
	assert_eq!(requests.len(), 1);

	let expect = format!(
		r#"IoctlRequest {{
    node_id: 2,
    handle: 1002,
    command: {tcgets2_cmd},
    arg: FAKE_ARG,
    output_len: {output_len},
    flags: IoctlRequestFlags {{}},
    input: [],
}}"#,
		tcgets2_cmd = format!("{:#010X}", libc::TCGETS2),
		output_len = size_of::<libc::termios2>(),
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn fuse_ioctl_tcsets2() {
	let termios2_buf = libc::termios2 {
		c_iflag: 12,
		c_oflag: 34,
		c_cflag: 56,
		c_lflag: 78,
		c_line: 0,
		c_cc: [0; 19],
		c_ispeed: 0,
		c_ospeed: 0,
	};

	let requests = fuse_ioctl_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let rc = unsafe {
			libc::ioctl(
				fd,
				libc::TCSETS2,
				&termios2_buf as *const libc::termios2,
			)
		};
		assert_eq!(rc, 0);

		unsafe { libc::close(fd) };

	});
	assert_eq!(requests.len(), 1);

	let input_bytes = unsafe {
		core::slice::from_raw_parts(
			(&termios2_buf as *const libc::termios2) as *const u8,
			size_of::<libc::termios2>(),
		)
	};
	let expect = format!(
		r#"IoctlRequest {{
    node_id: 2,
    handle: 1002,
    command: {tcsets2_cmd},
    arg: FAKE_ARG,
    output_len: 0,
    flags: IoctlRequestFlags {{}},
    input: {indented_input},
}}"#,
		tcsets2_cmd = format!("{:#010X}", libc::TCSETS2),
		indented_input = indented_input(input_bytes),
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
