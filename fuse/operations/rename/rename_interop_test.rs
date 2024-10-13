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

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	libc_errno,
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

		let cache_timeout = std::time::Duration::from_secs(60);

		if request.name() == "rename_old.txt" {
			let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(2).unwrap());
			attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
			attr.set_link_count(1);

			let mut entry = fuse::Entry::new(attr);
			entry.set_cache_timeout(cache_timeout);

			return send_reply.ok(&entry).unwrap();
		}

		if request.name() == "rename_new.txt" {
			let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(4).unwrap());
			attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
			attr.set_link_count(1);

			let mut entry = fuse::Entry::new(attr);
			entry.set_cache_timeout(cache_timeout);

			return send_reply.ok(&entry).unwrap();
		}

		if request.name() == "rename_dir.d" {
			let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(4).unwrap());
			attr.set_mode(fuse::FileMode::S_IFDIR | 0o755);
			attr.set_link_count(2);

			let mut entry = fuse::Entry::new(attr);
			entry.set_cache_timeout(cache_timeout);

			return send_reply.ok(&entry).unwrap();
		}

		send_reply.err(OsError::NOT_FOUND).unwrap();
	}

	fn rename(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::RenameRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();
		send_reply.ok_empty().unwrap();
	}

	fn rename2(&self, request: FuseRequest<'_>) {
		self.rename(request)
	}
}

fn rename_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

#[test]
fn rename() {
	let requests = rename_test(|root| {
		let path_src = path_cstr(root.join("rename_old.txt"));
		let path_dst = path_cstr(root.join("rename_new.txt"));

		let rc = unsafe { libc::rename(path_src.as_ptr(), path_dst.as_ptr()) };
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"RenameRequest {
    old_directory_id: 1,
    old_name: "rename_old.txt",
    new_directory_id: 1,
    new_name: "rename_new.txt",
    rename_flags: 0x00000000,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn rename_err_enoent() {
	let requests = rename_test(|root| {
		let path_src = path_cstr(root.join("rename_noexist.txt"));
		let path_dst = path_cstr(root.join("rename_new.txt"));

		let rc = unsafe { libc::rename(path_src.as_ptr(), path_dst.as_ptr()) };
		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::ENOENT);
	});
	assert_eq!(requests.len(), 0);
}

#[test]
fn rename_err_eisdir() {
	let requests = rename_test(|root| {
		let path_src = path_cstr(root.join("rename_old.txt"));
		let path_dst = path_cstr(root.join("rename_dir.d"));

		let rc = unsafe { libc::rename(path_src.as_ptr(), path_dst.as_ptr()) };
		assert_eq!(rc, -1);
		assert_eq!(libc_errno(), libc::EISDIR);
	});
	assert_eq!(requests.len(), 0);
}

#[test]
#[cfg(target_os = "linux")]
fn rename2_flag_exchange() {
	let requests = rename_test(|root| {
		let path_src = path_cstr(root.join("rename_old.txt"));
		let path_dst = path_cstr(root.join("rename_dir.d"));

		let rc = unsafe {
			libc::syscall(
				libc::SYS_renameat2,
				libc::AT_FDCWD,
				path_src.as_ptr(),
				libc::AT_FDCWD,
				path_dst.as_ptr(),
				libc::RENAME_EXCHANGE,
			)
		};
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"RenameRequest {
    old_directory_id: 1,
    old_name: "rename_old.txt",
    new_directory_id: 1,
    new_name: "rename_dir.d",
    rename_flags: 0x00000002,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn rename2_flag_noreplace() {
	let requests = rename_test(|root| {
		let path_src = path_cstr(root.join("rename_old.txt"));
		let path_dst = path_cstr(root.join("rename_noexist.txt"));

		let rc = unsafe {
			libc::syscall(
				libc::SYS_renameat2,
				libc::AT_FDCWD,
				path_src.as_ptr(),
				libc::AT_FDCWD,
				path_dst.as_ptr(),
				libc::RENAME_NOREPLACE,
			)
		};
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"RenameRequest {
    old_directory_id: 1,
    old_name: "rename_old.txt",
    new_directory_id: 1,
    new_name: "rename_noexist.txt",
    rename_flags: 0x00000001,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
#[cfg(target_os = "linux")]
fn rename2_flag_whiteout() {
	let requests = rename_test(|root| {
		let path_src = path_cstr(root.join("rename_old.txt"));
		let path_dst = path_cstr(root.join("rename_noexist.txt"));

		let rc = unsafe {
			libc::syscall(
				libc::SYS_renameat2,
				libc::AT_FDCWD,
				path_src.as_ptr(),
				libc::AT_FDCWD,
				path_dst.as_ptr(),
				libc::RENAME_WHITEOUT,
			)
		};
		assert_eq!(rc, 0);
	});
	assert_eq!(requests.len(), 1);

	let expect = r#"RenameRequest {
    old_directory_id: 1,
    old_name: "rename_old.txt",
    new_directory_id: 1,
    new_name: "rename_noexist.txt",
    rename_flags: 0x00000004,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
