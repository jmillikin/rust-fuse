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

use fuse::server::fuse_rpc;

use interop_testutil::{
	diff_str,
	errno,
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
		if request.parent_id() != fuse::ROOT_ID {
			return call.respond_err(ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		if request.name()
			== fuse::NodeName::from_bytes(b"rename_old.txt").unwrap()
		{
			node.set_id(fuse::NodeId::new(2).unwrap());

			let attr = node.attr_mut();
			attr.set_mode(fuse::FileType::Regular | 0o644);
			attr.set_nlink(1);

			return call.respond_ok(&resp);
		}

		if request.name()
			== fuse::NodeName::from_bytes(b"rename_new.txt").unwrap()
		{
			node.set_id(fuse::NodeId::new(3).unwrap());

			let attr = node.attr_mut();
			attr.set_mode(fuse::FileType::Regular | 0o644);
			attr.set_nlink(1);

			return call.respond_ok(&resp);
		}

		if request.name()
			== fuse::NodeName::from_bytes(b"rename_dir.d").unwrap()
		{
			node.set_id(fuse::NodeId::new(4).unwrap());

			let attr = node.attr_mut();
			attr.set_mode(fuse::FileType::Directory | 0o755);
			attr.set_nlink(2);

			return call.respond_ok(&resp);
		}

		call.respond_err(ErrorCode::ENOENT)
	}

	fn rename(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::RenameRequest,
	) -> fuse_rpc::FuseResult<fuse::RenameResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();
		let resp = fuse::RenameResponse::new();
		call.respond_ok(&resp)
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
		assert_eq!(errno(), libc::ENOENT);
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
		assert_eq!(errno(), libc::EISDIR);
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