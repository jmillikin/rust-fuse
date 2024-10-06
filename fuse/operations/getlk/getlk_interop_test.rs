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

use fuse::server::fuse_rpc;
use fuse::server::prelude::*;

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
	fn fuse_init_flags(flags: &mut FuseInitFlags) {
		flags.set(FuseInitFlag::POSIX_LOCKS);
	}
}

impl<S: FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &LookupRequest,
	) -> fuse_rpc::SendResult<LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let node_id;
		if request.name() == "getlk_u.txt" {
			node_id = fuse::NodeId::new(2).unwrap();
		} else if request.name() == "getlk_r.txt" {
			node_id = fuse::NodeId::new(3).unwrap();
		} else if request.name() == "getlk_w.txt" {
			node_id = fuse::NodeId::new(4).unwrap();
		} else {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut attr = fuse::Attributes::new(node_id);
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		request: &OpenRequest,
	) -> fuse_rpc::SendResult<OpenResponse, S::Error> {
		let mut resp = OpenResponse::new();
		resp.set_handle(1000 + request.node_id().get());
		call.respond_ok(&resp)
	}

	fn getlk(
		&self,
		call: fuse_rpc::Call<S>,
		request: &GetlkRequest,
	) -> fuse_rpc::SendResult<GetlkResponse, S::Error> {
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

		let range = fuse::LockRange::new(1024, num::NonZeroU64::new(3072));
		let pid = fuse::LockOwnerProcessId::new(std::process::id());

		let lock = if request.node_id() == fuse::NodeId::new(3).unwrap() {
			Some(fuse::Lock::new(fuse::LockMode::Shared, range, pid))
		} else if request.node_id() == fuse::NodeId::new(4).unwrap() {
			Some(fuse::Lock::new(fuse::LockMode::Exclusive, range, pid))
		} else {
			None
		};
		let resp = GetlkResponse::new(lock);
		call.respond_ok(&resp)
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
    lock_mode: Shared,
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
    lock_mode: Shared,
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
    lock_mode: Exclusive,
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
    lock_mode: Shared,
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
    lock_mode: Shared,
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
    lock_mode: Shared,
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
    lock_mode: Shared,
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
    lock_mode: Exclusive,
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
    lock_mode: Shared,
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
    lock_mode: Exclusive,
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
