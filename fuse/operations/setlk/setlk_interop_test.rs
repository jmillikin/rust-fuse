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
use fuse::{
	FuseInitFlag,
	FuseInitFlags,
};

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

	fn fuse_init_flags(flags: &mut FuseInitFlags) {
		flags.set(FuseInitFlag::FLOCK_LOCKS);
		flags.set(FuseInitFlag::POSIX_LOCKS);
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
		if request.name() != "setlk.txt" {
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

	fn setlk(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::SetlkRequest::try_from(request).unwrap();

		let mut request_str = format!("{:#?}", request);

		// stub out the lock owner, which is non-deterministic.
		let lock_owner = format!("owner: {:?},", request.owner());
		let repl_start = request_str.find(&lock_owner).unwrap();
		let repl_end = repl_start + lock_owner.len();
		request_str.replace_range(
			repl_start..repl_end,
			"owner: 123456789123456789,",
		);

		self.fs.requests.send(request_str).unwrap();
		send_reply.ok_empty().unwrap();
	}

	fn setlkw(&self, request: FuseRequest<'_>) {
		self.setlk(request)
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
    lock: Lock {{
        mode: F_RDLCK,
        range: LockRange {{
            start: 100,
            length: Some(50),
        }},
        process_id: Some({pid}),
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
		#[cfg(target_os = "freebsd")]
		let lock_pid_str = format!("Some({})", lock_pid);
		#[cfg(target_os = "linux")]
		let lock_pid_str = "None";
		let expect = format!(
			r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: Lock {{
        mode: F_UNLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: {pid},
    }},
    flags: SetlkRequestFlags {{}},
}}"#,
			pid = lock_pid_str,
		);
		if let Some(diff) = diff_str(&expect, &requests[1]) {
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
    lock: Lock {{
        mode: F_RDLCK,
        range: LockRange {{
            start: 100,
            length: Some(50),
        }},
        process_id: Some({pid}),
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
    lock: Lock {{
        mode: F_UNLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: None,
    }},
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
    lock: Lock {{
        mode: F_WRLCK,
        range: LockRange {{
            start: 100,
            length: Some(50),
        }},
        process_id: Some({pid}),
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
		#[cfg(target_os = "freebsd")]
		let lock_pid_str = format!("Some({})", lock_pid);
		#[cfg(target_os = "linux")]
		let lock_pid_str = "None";
		let expect = format!(
			r#"SetlkRequest {{
    node_id: 2,
    handle: 12345,
    may_block: false,
    owner: 123456789123456789,
    lock: Lock {{
        mode: F_UNLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: {pid},
    }},
    flags: SetlkRequestFlags {{}},
}}"#,
			pid = lock_pid_str,
		);
		if let Some(diff) = diff_str(&expect, &requests[1]) {
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
    lock: Lock {{
        mode: F_WRLCK,
        range: LockRange {{
            start: 100,
            length: Some(50),
        }},
        process_id: Some({pid}),
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
    lock: Lock {
        mode: F_UNLCK,
        range: LockRange {
            start: 100,
            length: Some(50),
        },
        process_id: None,
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
    lock: Lock {
        mode: F_UNLCK,
        range: LockRange {
            start: 100,
            length: Some(50),
        },
        process_id: None,
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
    lock: Lock {{
        mode: F_RDLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: Some({pid}),
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
    lock: Lock {{
        mode: F_RDLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: Some({pid}),
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
    lock: Lock {{
        mode: F_WRLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: Some({pid}),
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
    lock: Lock {{
        mode: F_WRLCK,
        range: LockRange {{
            start: 0,
            length: None,
        }},
        process_id: Some({pid}),
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
    lock: Lock {
        mode: F_UNLCK,
        range: LockRange {
            start: 0,
            length: None,
        },
        process_id: None,
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
    lock: Lock {
        mode: F_UNLCK,
        range: LockRange {
            start: 0,
            length: None,
        },
        process_id: None,
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
