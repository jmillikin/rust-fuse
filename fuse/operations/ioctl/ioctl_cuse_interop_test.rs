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

use fuse::server::cuse_rpc;
use fuse::server::prelude::*;

use interop_testutil::{diff_str, path_cstr, ErrorCode};

struct TestCharDev {
	requests: mpsc::Sender<String>,
}

impl interop_testutil::TestDev for TestCharDev {
	fn cuse_init_flags(flags: &mut CuseInitFlags) {
		flags.set(CuseInitFlag::UNRESTRICTED_IOCTL);
	}
}

impl<S: CuseSocket> cuse_rpc::Handlers<S> for TestCharDev {
	fn ioctl(
		&self,
		call: cuse_rpc::Call<S>,
		request: &IoctlRequest,
	) -> cuse_rpc::SendResult<IoctlResponse, S::Error> {
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

		if request.command().get() == libc::TIOCGWINSZ as u32 {
			if request.output_len() == 0 {
				let arg: IoctlPtr<libc::winsize> = request.arg().as_ptr();
				let mut retry = IoctlRetryBuf::new();
				retry.add_output_ptr(arg).unwrap();
				let resp = IoctlResponse::new_retry(retry.borrow());
				return call.respond_ok(&resp);
			}

			let winsize_buf = libc::winsize {
				ws_row: 123,
				ws_col: 456,
				ws_xpixel: 0,
				ws_ypixel: 0,
			};

			let bytes_1: &[u8] = unsafe {
				core::slice::from_raw_parts(
					(&winsize_buf as *const libc::winsize) as *const u8,
					size_of::<libc::winsize>(),
				)
			};

			let resp = IoctlResponse::new(bytes_1);
			return call.respond_ok(&resp);
		}

		if request.command().get() == libc::TIOCSWINSZ as u32 {
			if request.input_len() == 0 {
				let arg: IoctlPtr<libc::winsize> = request.arg().as_ptr();
				let mut retry = IoctlRetryBuf::new();
				retry.add_input_ptr(arg).unwrap();
				let resp = IoctlResponse::new_retry(retry.borrow());
				return call.respond_ok(&resp);
			}

			let resp = IoctlResponse::new(b"");
			return call.respond_ok(&resp);
		}

		call.respond_err(ErrorCode::EOPNOTSUPP)
	}

	fn open(
		&self,
		call: cuse_rpc::Call<S>,
		_request: &OpenRequest,
	) -> cuse_rpc::SendResult<OpenResponse, S::Error> {
		let mut resp = OpenResponse::new();
		resp.set_handle(1002);
		call.respond_ok(&resp)
	}
}

fn cuse_ioctl_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestCharDev {
		requests: request_send,
	};
	interop_testutil::cuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

fn indented_input(input: &[u8]) -> String {
	format!("{:#?}", input)
		.replace("[\n", "[\n    ")
		.replace(",\n ", ",\n     ")
		.replace("]", "    ]")
}

#[test]
fn cuse_ioctl_tiocgwinsz() {
	let requests = cuse_ioctl_test(|dev_path| {
		let path = path_cstr(dev_path.to_owned());

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut winsize_buf = libc::winsize {
			ws_row: 0,
			ws_col: 0,
			ws_xpixel: 0,
			ws_ypixel: 0,
		};
		let rc = unsafe {
			libc::ioctl(
				fd,
				libc::TIOCGWINSZ,
				&mut winsize_buf as *mut libc::winsize,
			)
		};
		assert_eq!(rc, 0);
		assert_eq!(winsize_buf.ws_row, 123);
		assert_eq!(winsize_buf.ws_col, 456);

		unsafe { libc::close(fd) };
	});
	assert_eq!(requests.len(), 2);

	let expect = r#"IoctlRequest {
    node_id: 1,
    handle: 1002,
    command: 0x00005413,
    arg: FAKE_ARG,
    output_len: 0,
    flags: IoctlRequestFlags {
        IOCTL_UNRESTRICTED,
    },
    input: [],
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}

	let expect = format!(
		r#"IoctlRequest {{
    node_id: 1,
    handle: 1002,
    command: 0x00005413,
    arg: FAKE_ARG,
    output_len: {output_len},
    flags: IoctlRequestFlags {{
        IOCTL_UNRESTRICTED,
    }},
    input: [],
}}"#,
		output_len = size_of::<libc::winsize>(),
	);
	if let Some(diff) = diff_str(&expect, &requests[1]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn cuse_ioctl_tiocswinsz() {
	let winsize_buf = libc::winsize {
		ws_row: 123,
		ws_col: 456,
		ws_xpixel: 0,
		ws_ypixel: 0,
	};

	let requests = cuse_ioctl_test(|dev_path| {
		let path = path_cstr(dev_path.to_owned());

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let rc = unsafe {
			libc::ioctl(
				fd,
				libc::TIOCSWINSZ,
				&winsize_buf as *const libc::winsize,
			)
		};
		assert_eq!(rc, 0);

		unsafe { libc::close(fd) };
	});
	assert_eq!(requests.len(), 2);

	let expect = r#"IoctlRequest {
    node_id: 1,
    handle: 1002,
    command: 0x00005414,
    arg: FAKE_ARG,
    output_len: 0,
    flags: IoctlRequestFlags {
        IOCTL_UNRESTRICTED,
    },
    input: [],
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}

	let input_bytes = unsafe {
		core::slice::from_raw_parts(
			(&winsize_buf as *const libc::winsize) as *const u8,
			size_of::<libc::winsize>(),
		)
	};
	let expect = format!(
		r#"IoctlRequest {{
    node_id: 1,
    handle: 1002,
    command: 0x00005414,
    arg: FAKE_ARG,
    output_len: 0,
    flags: IoctlRequestFlags {{
        IOCTL_UNRESTRICTED,
    }},
    input: {indented_input},
}}"#,
		indented_input = indented_input(input_bytes),
	);
	if let Some(diff) = diff_str(&expect, &requests[1]) {
		println!("{}", diff);
		assert!(false);
	}
}
