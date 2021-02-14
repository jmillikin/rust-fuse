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

use core::num::NonZeroU64;
use std::sync::mpsc;
use std::{fmt, panic};

use interop_testutil::{diff_str, interop_test, path_cstr};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl fuse::FuseHandlers for TestFS {
	fn fuse_init(
		&mut self,
		_request: &fuse::FuseInitRequest,
	) -> fuse::FuseInitResponse {
		let mut resp = fuse::FuseInitResponse::new();
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

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		if request.name() == fuse::NodeName::from_bytes(b"getlk_u.txt").unwrap()
		{
			node.set_id(fuse::NodeId::new(2).unwrap());
		} else if request.name()
			== fuse::NodeName::from_bytes(b"getlk_r.txt").unwrap()
		{
			node.set_id(fuse::NodeId::new(3).unwrap());
		} else if request.name()
			== fuse::NodeName::from_bytes(b"getlk_w.txt").unwrap()
		{
			node.set_id(fuse::NodeId::new(4).unwrap());
		} else {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(2);

		respond.ok(&resp);
	}

	fn open(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpenRequest,
		respond: impl for<'a> fuse::Respond<fuse::OpenResponse<'a>>,
	) {
		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(1000 + request.node_id().get());
		respond.ok(&resp);
	}

	fn getlk(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::GetlkRequest,
		respond: impl for<'a> fuse::Respond<fuse::GetlkResponse<'a>>,
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

		let mut resp = fuse::GetlkResponse::new();

		let range = fuse::LockRange::new(1024, NonZeroU64::new(3072));
		if request.node_id() == fuse::NodeId::new(3).unwrap() {
			let mut lock = fuse::Lock::new_shared(range);
			lock.set_process_id(123);
			resp.set_lock(Some(lock));
		} else if request.node_id() == fuse::NodeId::new(4).unwrap() {
			let mut lock = fuse::Lock::new_exclusive(range);
			lock.set_process_id(123);
			resp.set_lock(Some(lock));
		} else {
			resp.set_lock(None);
		}
		respond.ok(&resp);
	}
}

fn getlk_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	interop_test(fs, test_fn);
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

#[allow(unreachable_code, unused_variables)]
fn lock_pid(pid: u32) -> u32 {
	#[cfg(target_os = "linux")]
	return 0;
	pid
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
				l_pid: 3000,
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
			l_pid: 3000,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock: Shared {{
        range: 100..150,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: 50,
			l_pid: 3000,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock: Exclusive {{
        range: 100..150,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: 0,
			l_pid: 3000,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock: Shared {{
        range: 100..,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 0,
			l_len: 1,
			l_pid: 3000,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock: Shared {{
        range: 0..1,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_UNLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 100,
			l_len: -50,
			l_pid: 3000,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 2,
    handle: 1002,
    owner: 123456789123456789,
    lock: Shared {{
        range: 50..100,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_RDLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: 123,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		#[cfg(target_os = "linux")]
		{
			expect_lock.l_whence = libc::SEEK_SET as i16;
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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 3,
    handle: 1003,
    owner: 123456789123456789,
    lock: Shared {{
        range: 100..150,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_RDLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: 123,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		#[cfg(target_os = "linux")]
		{
			expect_lock.l_whence = libc::SEEK_SET as i16;
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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 3,
    handle: 1003,
    owner: 123456789123456789,
    lock: Exclusive {{
        range: 100..150,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_WRLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: 123,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		#[cfg(target_os = "linux")]
		{
			expect_lock.l_whence = libc::SEEK_SET as i16;
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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 4,
    handle: 1004,
    owner: 123456789123456789,
    lock: Shared {{
        range: 100..150,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
				l_pid: 3000,
				#[cfg(target_os = "freebsd")]
				l_sysid: 400,
			},
		);

		#[allow(unused_mut)]
		let mut expect_lock = libc::flock {
			l_type: libc::F_WRLCK as i16,
			l_whence: libc::SEEK_CUR as i16,
			l_start: 1024,
			l_len: 3072,
			l_pid: 123,
			#[cfg(target_os = "freebsd")]
			l_sysid: 0,
		};

		#[cfg(target_os = "linux")]
		{
			expect_lock.l_whence = libc::SEEK_SET as i16;
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

	let expect = format!(
		r#"GetlkRequest {{
    node_id: 4,
    handle: 1004,
    owner: 123456789123456789,
    lock: Exclusive {{
        range: 100..150,
        process_id: {pid},
    }},
}}"#,
		pid = lock_pid(3000)
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
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
