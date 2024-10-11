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

use core::num;
use std::sync::mpsc;
use std::{fmt, panic};

use fuse::{
	FuseInitFlag,
	FuseInitFlags,
};
use fuse::server;
use fuse::server::FuseRequest;

#[cfg(target_os = "freebsd")]
use fuse::os::freebsd::{F_RDLCK, F_WRLCK, F_UNLCK};

#[cfg(target_os = "linux")]
use fuse::os::linux::{F_RDLCK, F_WRLCK, F_UNLCK};

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

		let node_id;
		if request.name() == "getlk_u.txt" {
			node_id = fuse::NodeId::new(2).unwrap();
		} else if request.name() == "getlk_r.txt" {
			node_id = fuse::NodeId::new(3).unwrap();
		} else if request.name() == "getlk_w.txt" {
			node_id = fuse::NodeId::new(4).unwrap();
		} else {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::Attributes::new(node_id);
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn open(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 1000 + request.header().raw().nodeid;
		send_reply.ok(&reply).unwrap();
	}

	fn getlk(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::GetlkRequest::try_from(request).unwrap();
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

		let range = fuse::LockRange::new(1024, num::NonZeroU64::new(3072));
		let pid = fuse::LockOwnerProcessId::new(std::process::id());

		let lock = if request.node_id() == fuse::NodeId::new(3).unwrap() {
			fuse::Lock::new(F_RDLCK, range, pid)
		} else if request.node_id() == fuse::NodeId::new(4).unwrap() {
			fuse::Lock::new(F_WRLCK, range, pid)
		} else {
			fuse::Lock::new(F_UNLCK, fuse::LockRange::new(0, None), None)
		};

		// FIXME
		let mut reply = fuse::kernel::fuse_lk_out::new();
		reply.lk.start = lock.range().start();
		reply.lk.end = lock.range().end().unwrap_or(i64::MAX as u64);
		reply.lk.r#type = lock.mode().0;
		if lock.mode() != F_UNLCK {
			reply.lk.pid = pid.unwrap().get();
		}

		send_reply.ok(&reply).unwrap();
	}
}

fn getlk_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

fn fcntl_getlk(path: std::path::PathBuf, mut lock: libc::flock) -> libc::flock {
	let path_cstr = path_cstr(path);

	let file_fd = unsafe { libc::open(path_cstr.as_ptr(), libc::O_RDWR) };
	assert_ne!(file_fd, -1);
	let rc = unsafe { libc::fcntl(file_fd, libc::F_GETLK, &mut lock) };
	unsafe {
		libc::close(file_fd)
	};
	assert_eq!(rc, 0);

	lock
}

#[cfg(target_os = "linux")]
fn fcntl_ofd_getlk(
	path: std::path::PathBuf,
	mut lock: libc::flock,
) -> libc::flock {
	let path_cstr = path_cstr(path);

	let file_fd = unsafe { libc::open(path_cstr.as_ptr(), libc::O_RDWR) };
	assert_ne!(file_fd, -1);
	let rc = unsafe { libc::fcntl(file_fd, libc::F_OFD_GETLK, &mut lock) };
	unsafe {
		libc::close(file_fd)
	};
	assert_eq!(rc, 0);

	lock
}

#[test]
fn getlk_fcntl_read_unlocked() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_u.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: 50,
			l_pid: 9999,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266885
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_pid = 0;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[cfg(target_os = "linux")]
#[test]
fn getlk_fcntl_read_unlocked_ofd() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_u.txt");

		let got_lock = fcntl_ofd_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 0,
			},
		);

		let expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: 50,
			l_pid: 0,
		};

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_write_unlocked() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_u.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_WRLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: 50,
			l_pid: 9999,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266885
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_pid = 0;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock_mode: F_WRLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_read_unlocked_zero_len() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_u.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 0,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: 0,
			l_pid: 9999,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266885
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_pid = 0;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 100,
        length: None,
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_read_unlocked_one_byte() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_u.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 0,
				l_len: 1,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 0,
			l_len: 1,
			l_pid: 9999,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266885
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_pid = 0;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 0,
        length: Some(1),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_read_unlocked_negative_len() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_u.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: -50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: -50,
			l_pid: 9999,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266885
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_pid = 0;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 50,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_read_locked_r() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_r.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_RDLCK as i16,
			l_whence: libc::SEEK_SET as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: std::process::id() as i32,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266886
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_whence = libc::SEEK_CUR as i16;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 3,
    handle: 1003,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_write_locked_r() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_r.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_WRLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_RDLCK as i16,
			l_whence: libc::SEEK_SET as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: std::process::id() as i32,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266886
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_whence = libc::SEEK_CUR as i16;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 3,
    handle: 1003,
    owner: 123456789123456789,
    lock_mode: F_WRLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_read_locked_w() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_w.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_RDLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_WRLCK as i16,
			l_whence: libc::SEEK_SET as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: std::process::id() as i32,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266886
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_whence = libc::SEEK_CUR as i16;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 4,
    handle: 1004,
    owner: 123456789123456789,
    lock_mode: F_RDLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn getlk_fcntl_write_locked_w() {
	let requests = getlk_test(|root| {
		let path = root.join("getlk_w.txt");

		let got_lock = fcntl_getlk(
			path,
			libc::flock {
				l_type: libc::F_WRLCK as i16,
				l_whence: libc::SEEK_CUR as i16,
				l_start: 100,
				l_len: 50,
				l_pid: 9999,
				#[cfg(target_os = "freebsd")]
				l_sysid: 0,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_WRLCK as i16,
			l_whence: libc::SEEK_SET as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: std::process::id() as i32,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=266886
		#[cfg(target_os = "freebsd")]
		{
			expect_lock.l_whence = libc::SEEK_CUR as i16;
		}

		let expect = format!("{:#?}", &DebugFlock(expect_lock));
		let got = format!("{:#?}", &DebugFlock(got_lock));
		if let Some(diff) = diff_str(&expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"GetlkRequest {
    node_id: 4,
    handle: 1004,
    owner: 123456789123456789,
    lock_mode: F_WRLCK,
    lock_range: LockRange {
        start: 100,
        length: Some(50),
    },
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

struct DebugFlock(libc::flock);

impl fmt::Debug for DebugFlock {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("flock")
			.field("l_type", &self.0.l_type)
			.field("l_whence", &self.0.l_whence)
			.field("l_start", &self.0.l_start)
			.field("l_len", &self.0.l_len)
			.field("l_pid", &self.0.l_pid)
			.finish()
	}
}
