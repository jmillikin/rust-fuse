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

use interop_testutil::{diff_str, fuse_interop_test, path_cstr};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl fuse::FuseHandlers for TestFS {
	fn fuse_init(
		&mut self,
		_request: &fuse::FuseInitRequest,
	) -> fuse::FuseInitResponse {
		let mut resp = fuse::FuseInitResponse::new();
		resp.flags_mut().flock_locks = true;
		resp.flags_mut().posix_locks = true;
		resp
	}

	fn lookup(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::LookupRequest,
		respond: impl for<'a> fuse::Respond<fuse::LookupResponse<'a>>,
	) {
		if request.parent_id() != fuse::ROOT_ID {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}
		if request.name() != fuse::NodeName::from_bytes(b"setlk.txt").unwrap() {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(2);

		respond.ok(&resp);
	}

	fn open(
		&self,
		_ctx: fuse::ServerContext,
		_request: &fuse::OpenRequest,
		respond: impl for<'a> fuse::Respond<fuse::OpenResponse<'a>>,
	) {
		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(12345);
		respond.ok(&resp);
	}

	fn setlk(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::SetlkRequest,
		respond: impl for<'a> fuse::Respond<fuse::SetlkResponse<'a>>,
	) {
		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let lock_owner = format!("owner: {},", request.owner());
		let repl_start = request_str.find(&lock_owner).unwrap();
		let repl_end = repl_start + lock_owner.len();
		request_str.replace_range(
			repl_start..repl_end,
			"owner: 123456789123456789,",
		);

		self.requests.send(request_str).unwrap();

		let resp = fuse::SetlkResponse::new();
		respond.ok(&resp);
	}
}

fn setlk_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

fn fcntl_setlk(path: std::path::PathBuf, mut lock: libc::flock) {
	let path_cstr = path_cstr(path);

	let file_fd = unsafe { libc::open(path_cstr.as_ptr(), libc::O_RDWR) };
	assert_ne!(file_fd, -1);
	let rc = unsafe { libc::fcntl(file_fd, libc::F_SETLK, &mut lock) };
	unsafe {
		libc::close(file_fd)
	};
	assert_eq!(rc, 0);
}

fn fcntl_setlkw(path: std::path::PathBuf, mut lock: libc::flock) {
	let path_cstr = path_cstr(path);

	let file_fd = unsafe { libc::open(path_cstr.as_ptr(), libc::O_RDWR) };
	assert_ne!(file_fd, -1);
	let rc = unsafe { libc::fcntl(file_fd, libc::F_SETLKW, &mut lock) };
	unsafe {
		libc::close(file_fd)
	};
	assert_eq!(rc, 0);
}

#[cfg(not(target_os = "freebsd"))]
fn flock(path: std::path::PathBuf, operation: i32) {
	let path_cstr = path_cstr(path);

	let file_fd = unsafe { libc::open(path_cstr.as_ptr(), libc::O_RDWR) };
	assert_ne!(file_fd, -1);
	let rc = unsafe { libc::flock(file_fd, operation) };
	unsafe {
		libc::close(file_fd)
	};
	assert_eq!(rc, 0);
}

#[test]
#[cfg(not(target_os = "freebsd"))] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlk_fcntl_read() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlkw(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_SET as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	#[cfg(target_os = "freebsd")]
	let lock_pid = 3000;

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: SetLock(
        Shared {{
            range: 100..150,
            process_id: {pid},
        }},
    ),
    flags: SetlkRequestFlags {{
        flock: false,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn setlk_fcntl_read_nonblocking() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	#[cfg(target_os = "freebsd")]
	let lock_pid = 3000;

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: TrySetLock(
        Shared {{
            range: 100..150,
            process_id: {pid},
        }},
    ),
    flags: SetlkRequestFlags {{
        flock: false,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(not(target_os = "freebsd"))] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlk_fcntl_write() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlkw(
			path,
			libc::flock {
				l_type: libc::F_WRLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	#[cfg(target_os = "freebsd")]
	let lock_pid = 3000;

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: SetLock(
        Exclusive {{
            range: 100..150,
            process_id: {pid},
        }},
    ),
    flags: SetlkRequestFlags {{
        flock: false,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(not(target_os = "freebsd"))] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlk_fcntl_unlock() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlkw(
			path,
			libc::flock {
				l_type: libc::F_UNLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = 0;

	#[cfg(target_os = "freebsd")]
	let lock_pid = 3000;

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: ClearLocks {{
        range: 100..150,
        process_id: {pid},
    }},
    flags: SetlkRequestFlags {{
        flock: false,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(not(target_os = "freebsd"))]
fn setlk_flock_shared() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_SH);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: SetLock(
        Shared {{
            range: 0..,
            process_id: {pid},
        }},
    ),
    flags: SetlkRequestFlags {{
        flock: true,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(not(target_os = "freebsd"))]
fn setlk_flock_exclusive() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_EX);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: SetLock(
        Exclusive {{
            range: 0..,
            process_id: {pid},
        }},
    ),
    flags: SetlkRequestFlags {{
        flock: true,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(not(target_os = "freebsd"))]
fn setlk_flock_shared_nonblocking() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_SH | libc::LOCK_NB);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: TrySetLock(
        Shared {{
            range: 0..,
            process_id: {pid},
        }},
    ),
    flags: SetlkRequestFlags {{
        flock: true,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(not(target_os = "freebsd"))]
fn setlk_flock_unlock() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_UN);
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let lock_pid = 0;

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    owner: 123456789123456789,
    command: ClearLocks {{
        range: 0..,
        process_id: {pid},
    }},
    flags: SetlkRequestFlags {{
        flock: true,
    }},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
