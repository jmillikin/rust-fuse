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

impl<S: fuse_rpc::FuseSocket> fuse_rpc::FuseHandlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "file.txt" {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(node::Id::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(node::Mode::S_IFREG | 0o644);
		attr.set_nlink(1);

		call.respond_ok(&resp)
	}

	fn open(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::OpenRequest,
	) -> fuse_rpc::FuseResult<fuse::OpenResponse, S::Error> {
		if request.node_id().get() != 2 {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(10);
		call.respond_ok(&resp)
	}

	#[allow(non_snake_case)]
	fn poll(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::PollRequest,
	) -> fuse_rpc::FuseResult<fuse::PollResponse, S::Error> {
		let mut request_str = format!("{:#?}", request);

		// stub out the poll handle, which is non-deterministic.
		let poll_handle = format!("poll_handle: {:?},", request.poll_handle());
		let repl_start = request_str.find(&poll_handle).unwrap();
		let repl_end = repl_start + poll_handle.len();
		request_str.replace_range(
			repl_start..repl_end,
			"poll_handle: STUB_POLL_HANDLE,",
		);

		self.requests.send(request_str).unwrap();

		let POLLIN = libc::POLLIN as u32;
		let POLLOUT = libc::POLLOUT as u32;

		let mut resp = fuse::PollResponse::new();
		if (request.poll_events() & POLLIN) > 0 {
			resp.set_poll_events(POLLIN);
		}
		if (request.poll_events() & POLLOUT) > 0 {
			resp.set_poll_events(POLLOUT);
		}
		call.respond_ok(&resp)
	}

	fn release(
		&self,
		call: fuse_rpc::FuseCall<S>,
		_request: &fuse::ReleaseRequest,
	) -> fuse_rpc::FuseResult<fuse::ReleaseResponse, S::Error> {
		let resp = fuse::ReleaseResponse::new();
		call.respond_ok(&resp)
	}
}

fn poll_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

fn fd_set_new() -> libc::fd_set {
	let mut fd_set = std::mem::MaybeUninit::<libc::fd_set>::uninit();
	unsafe {
		libc::FD_ZERO(fd_set.as_mut_ptr());
		fd_set.assume_init()
	}
}

#[test]
fn test_poll_read() {
	let requests = poll_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut poll_fds = [
			libc::pollfd {
				fd,
				events: libc::POLLIN,
				revents: 0,
			},
		];

		let rc = unsafe {
			libc::poll(
				poll_fds.as_mut_ptr(),
				poll_fds.len() as libc::nfds_t,
				1000,
			)
		};
		assert_ne!(rc, -1);

		unsafe { libc::close(fd) };
	});
	assert!(requests.len() > 0);

	let poll_events =
		  libc::POLLIN
		| libc::POLLERR
		| libc::POLLHUP
	;
	let expect = format!(r#"PollRequest {{
    node_id: 2,
    poll_handle: STUB_POLL_HANDLE,
    poll_events: {poll_events:#010X},
    flags: PollRequestFlags {{
        SCHEDULE_NOTIFY,
    }},
}}"#);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn test_poll_write() {
	let requests = poll_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut poll_fds = [
			libc::pollfd {
				fd,
				events: libc::POLLOUT,
				revents: 0,
			},
		];

		let rc = unsafe {
			libc::poll(
				poll_fds.as_mut_ptr(),
				poll_fds.len() as libc::nfds_t,
				1000,
			)
		};
		assert_ne!(rc, -1);

		unsafe { libc::close(fd) };
	});
	assert!(requests.len() > 0);

	let poll_events =
		  libc::POLLOUT
		| libc::POLLERR
		| libc::POLLHUP
	;
	let expect = format!(r#"PollRequest {{
    node_id: 2,
    poll_handle: STUB_POLL_HANDLE,
    poll_events: {poll_events:#010X},
    flags: PollRequestFlags {{
        SCHEDULE_NOTIFY,
    }},
}}"#);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn test_poll_except() {
	let requests = poll_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut poll_fds = [
			libc::pollfd {
				fd,
				events: 0,
				revents: 0,
			},
		];

		let rc = unsafe {
			libc::poll(
				poll_fds.as_mut_ptr(),
				poll_fds.len() as libc::nfds_t,
				1000,
			)
		};
		assert_ne!(rc, -1);

		unsafe { libc::close(fd) };
	});
	assert!(requests.len() > 0);

	let poll_events =
		  libc::POLLERR
		| libc::POLLHUP
	;
	let expect = format!(r#"PollRequest {{
    node_id: 2,
    poll_handle: STUB_POLL_HANDLE,
    poll_events: {poll_events:#010X},
    flags: PollRequestFlags {{
        SCHEDULE_NOTIFY,
    }},
}}"#);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn test_select_read() {
	let requests = poll_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut fd_set = fd_set_new();
		unsafe { libc::FD_SET(fd, &mut fd_set) };

		let mut timeout = libc::timeval {
			tv_sec: 1,
			tv_usec: 0,
		};

		let rc = unsafe {
			libc::select(
				fd + 1,
				&mut fd_set,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				&mut timeout,
			)
		};
		assert_ne!(rc, -1);

		unsafe { libc::close(fd) };
	});
	assert!(requests.len() > 0);

	let poll_events =
		  libc::POLLIN
		| libc::POLLPRI
		| libc::POLLERR
		| libc::POLLHUP
		| libc::POLLNVAL
		| libc::POLLRDNORM
		| libc::POLLRDBAND
	;
	let expect = format!(r#"PollRequest {{
    node_id: 2,
    poll_handle: STUB_POLL_HANDLE,
    poll_events: {poll_events:#010X},
    flags: PollRequestFlags {{
        SCHEDULE_NOTIFY,
    }},
}}"#);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn test_select_write() {
	let requests = poll_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut fd_set = fd_set_new();
		unsafe { libc::FD_SET(fd, &mut fd_set) };

		let mut timeout = libc::timeval {
			tv_sec: 1,
			tv_usec: 0,
		};

		let rc = unsafe {
			libc::select(
				fd + 1,
				std::ptr::null_mut(),
				&mut fd_set,
				std::ptr::null_mut(),
				&mut timeout,
			)
		};
		assert_ne!(rc, -1);

		unsafe { libc::close(fd) };
	});
	assert!(requests.len() > 0);

	let poll_events =
		  libc::POLLPRI
		| libc::POLLOUT
		| libc::POLLERR
		| libc::POLLNVAL
		| libc::POLLWRNORM
		| libc::POLLWRBAND
	;
	let expect = format!(r#"PollRequest {{
    node_id: 2,
    poll_handle: STUB_POLL_HANDLE,
    poll_events: {poll_events:#010X},
    flags: PollRequestFlags {{
        SCHEDULE_NOTIFY,
    }},
}}"#);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn test_select_except() {
	let requests = poll_test(|root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
		assert_ne!(fd, -1);

		let mut fd_set = fd_set_new();
		unsafe { libc::FD_SET(fd, &mut fd_set) };

		let mut timeout = libc::timeval {
			tv_sec: 1,
			tv_usec: 0,
		};

		let rc = unsafe {
			libc::select(
				fd + 1,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				&mut fd_set,
				&mut timeout,
			)
		};
		assert_ne!(rc, -1);

		unsafe { libc::close(fd) };
	});
	assert!(requests.len() > 0);

	let poll_events =
		  libc::POLLPRI
		| libc::POLLNVAL
	;
	let expect = format!(r#"PollRequest {{
    node_id: 2,
    poll_handle: STUB_POLL_HANDLE,
    poll_events: {poll_events:#010X},
    flags: PollRequestFlags {{
        SCHEDULE_NOTIFY,
    }},
}}"#);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
