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

use fuse::server;
use fuse::server::FuseRequest;

use interop_testutil::{
	diff_str,
	errno,
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
		if request.name() != "file.txt" {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::Attributes::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn getattr(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::GetattrRequest::try_from(request).unwrap();
		println!("{:#?}", request);

		let mut attr = fuse::Attributes::new(request.node_id());

		if request.node_id().is_root() {
			attr.set_mode(fuse::FileMode::S_IFDIR | 0o755);
			attr.set_link_count(2);
			let mut reply = fuse::kernel::fuse_attr_out::new();
			reply.attr = *attr.raw();
			return send_reply.ok(&reply).unwrap();
		}

		if request.node_id() == fuse::NodeId::new(2).unwrap() {
			attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
			attr.set_link_count(1);
			let mut reply = fuse::kernel::fuse_attr_out::new();
			reply.attr = *attr.raw();
			return send_reply.ok(&reply).unwrap();
		}

		send_reply.err(OsError::NOT_FOUND).unwrap();
	}

	fn ioctl(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::IoctlRequest::try_from(request).unwrap();
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

		self.fs.requests.send(request_str).unwrap();

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
			let resp = server::IoctlResponse::new(response_buf);
			return send_reply.ok(&resp).unwrap();
		}

		if request.command().get() == libc::TCSETS2 as u32 {
			let resp = server::IoctlResponse::new(b"");
			return send_reply.ok(&resp).unwrap();
		}

		send_reply.err(OsError(errno::EOPNOTSUPP)).unwrap();
	}

	fn open(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::OpenRequest::try_from(request).unwrap();
		println!("{:#?}", request);
		let mut reply = fuse::kernel::fuse_open_out::new();
		if request.node_id() == fuse::NodeId::new(2).unwrap() {
			reply.fh = 1002;
			return send_reply.ok(&reply).unwrap();
		}
		send_reply.err(OsError::NOT_FOUND).unwrap();
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
