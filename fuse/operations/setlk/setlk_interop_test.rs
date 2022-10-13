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

impl interop_testutil::TestFS for TestFS {
	fn fuse_init(
		_request: &fuse::FuseInitRequest,
		response: &mut fuse::FuseInitResponse,
	) {
		response.mut_flags().set(fuse::FuseInitFlag::FLOCK_LOCKS);
		response.mut_flags().set(fuse::FuseInitFlag::POSIX_LOCKS);
	}
}

impl<S: fuse_rpc::FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "setlk.txt" {
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

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		_request: &fuse::OpenRequest,
	) -> fuse_rpc::FuseResult<fuse::OpenResponse, S::Error> {
		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(12345);
		call.respond_ok(&resp)
	}

	fn setlk(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::SetlkRequest,
	) -> fuse_rpc::FuseResult<fuse::SetlkResponse, S::Error> {
		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let lock_owner = format!("owner: {:?},", request.owner());
		let repl_start = request_str.find(&lock_owner).unwrap();
		let repl_end = repl_start + lock_owner.len();
		request_str.replace_range(
			repl_start..repl_end,
			"owner: 123456789123456789,",
		);

		self.requests.send(request_str).unwrap();

		let resp = fuse::SetlkResponse::new();
		call.respond_ok(&resp)
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
fn setlk_fcntl_read() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlk(
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

	#[cfg(target_os = "linux")]
	let stlk_request_count = 1;

	#[cfg(target_os = "freebsd")]
	let stlk_request_count = 2;

	assert_eq!(requests.len(), stlk_request_count);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Shared,
            range: Range {{
                start: 100,
                length: Some(50),
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 100,
        length: Some(50),
    }},
    flags: SetlkRequestFlags {{}},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}

	if stlk_request_count >= 2 {
		let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 0,
        length: None,
    },
    flags: SetlkRequestFlags {},
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
#[cfg_attr(target_os = "freebsd", ignore)] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlkw_fcntl_read() {
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

	#[cfg(target_os = "linux")]
	let stlk_request_count = 1;

	#[cfg(target_os = "freebsd")]
	let stlk_request_count = 2;

	assert_eq!(requests.len(), stlk_request_count);

	#[cfg(target_os = "linux")]
	let lock_pid = std::process::id();

	#[cfg(target_os = "freebsd")]
	let lock_pid = 3000;

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: true,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Shared,
            range: Range {{
                start: 100,
                length: Some(50),
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 100,
        length: Some(50),
    }},
    flags: SetlkRequestFlags {{}},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}

	if stlk_request_count >= 2 {
		let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 0,
        length: None,
    },
    flags: SetlkRequestFlags {},
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
fn setlk_fcntl_write() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlk(
			path,
			libc::flock {
				l_type: libc::F_WRLCK as i16,
				l_whence: libc::SEEK_SET as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);
	});

	#[cfg(target_os = "linux")]
	let stlk_request_count = 1;

	#[cfg(target_os = "freebsd")]
	let stlk_request_count = 2;

	assert_eq!(requests.len(), stlk_request_count);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Exclusive,
            range: Range {{
                start: 100,
                length: Some(50),
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 100,
        length: Some(50),
    }},
    flags: SetlkRequestFlags {{}},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}

	if stlk_request_count >= 2 {
		let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 0,
        length: None,
    },
    flags: SetlkRequestFlags {},
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[test]
#[cfg_attr(target_os = "freebsd", ignore)] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlkw_fcntl_write() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlkw(
			path,
			libc::flock {
				l_type: libc::F_WRLCK as i16,
				l_whence: libc::SEEK_SET as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: true,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Exclusive,
            range: Range {{
                start: 100,
                length: Some(50),
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 100,
        length: Some(50),
    }},
    flags: SetlkRequestFlags {{}},
}}"#,
		pid = lock_pid
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg_attr(target_os = "freebsd", ignore)] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlk_fcntl_unlock() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlk(
			path,
			libc::flock {
				l_type: libc::F_UNLCK as i16,
				l_whence: libc::SEEK_SET as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 100,
        length: Some(50),
    },
    flags: SetlkRequestFlags {},
}"#;
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg_attr(target_os = "freebsd", ignore)] // https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253500
fn setlkw_fcntl_unlock() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		fcntl_setlkw(
			path,
			libc::flock {
				l_type: libc::F_UNLCK as i16,
				l_whence: libc::SEEK_SET as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: true,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 100,
        length: Some(50),
    },
    flags: SetlkRequestFlags {},
}"#;
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg_attr(target_os = "freebsd", ignore)] // FUSE_LK_FLOCK not supported by FreeBSD
fn setlk_flock_shared() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_SH | libc::LOCK_NB);
	});
	assert_eq!(requests.len(), 1);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Shared,
            range: Range {{
                start: 0,
                length: None,
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 0,
        length: None,
    }},
    flags: SetlkRequestFlags {{
        LK_FLOCK,
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
#[cfg_attr(target_os = "freebsd", ignore)] // FUSE_LK_FLOCK not supported by FreeBSD
fn setlkw_flock_shared() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_SH);
	});
	assert_eq!(requests.len(), 1);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: true,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Shared,
            range: Range {{
                start: 0,
                length: None,
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 0,
        length: None,
    }},
    flags: SetlkRequestFlags {{
        LK_FLOCK,
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
#[cfg_attr(target_os = "freebsd", ignore)] // FUSE_LK_FLOCK not supported by FreeBSD
fn setlk_flock_exclusive() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_EX | libc::LOCK_NB);
	});
	assert_eq!(requests.len(), 1);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Exclusive,
            range: Range {{
                start: 0,
                length: None,
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 0,
        length: None,
    }},
    flags: SetlkRequestFlags {{
        LK_FLOCK,
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
#[cfg_attr(target_os = "freebsd", ignore)] // FUSE_LK_FLOCK not supported by FreeBSD
fn setlkw_flock_exclusive() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_EX);
	});
	assert_eq!(requests.len(), 1);

	let lock_pid = std::process::id();

	let expect = format!(
		r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: true,
    owner: 123456789123456789,
    lock: Some(
        Lock {{
            mode: Exclusive,
            range: Range {{
                start: 0,
                length: None,
            }},
            process_id: Some({pid}),
        }},
    ),
    lock_range: Range {{
        start: 0,
        length: None,
    }},
    flags: SetlkRequestFlags {{
        LK_FLOCK,
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
#[cfg_attr(target_os = "freebsd", ignore)] // FUSE_LK_FLOCK not supported by FreeBSD
fn setlk_flock_unlock() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_UN | libc::LOCK_NB);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 0,
        length: None,
    },
    flags: SetlkRequestFlags {
        LK_FLOCK,
    },
}"#;
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg_attr(target_os = "freebsd", ignore)] // FUSE_LK_FLOCK not supported by FreeBSD
fn setlkw_flock_unlock() {
	let requests = setlk_test(|root| {
		let path = root.join("setlk.txt");
		flock(path, libc::LOCK_UN);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"SetlkRequest {
    node_id: 2,
    handle: 12345,
    may_block: true,
    owner: 123456789123456789,
    lock: None,
    lock_range: Range {
        start: 0,
        length: None,
    },
    flags: SetlkRequestFlags {
        LK_FLOCK,
    },
}"#;
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
