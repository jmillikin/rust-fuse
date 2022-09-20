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

use std::sync::mpsc;
use std::{fmt, mem, panic};

mod fuse {
	pub use ::fuse::*;
	pub use ::fuse::io::*;
	pub use ::fuse::protocol::*;
	pub use ::fuse::server::basic::*;

	pub use interop_testutil::ErrorCode;
}

use interop_testutil::{diff_str, fuse_interop_test, path_cstr};

struct TestFS {
	requests: mpsc::Sender<String>,
}

impl interop_testutil::TestFS for TestFS {}

impl<S: fuse::ServerSocket> fuse::FuseHandlers<S> for TestFS {
	fn lookup(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::LookupRequest,
		send_reply: impl fuse::SendReply<S>,
	) -> fuse::SendResult<fuse::LookupResponse, S::Error> {
		if request.parent_id() != fuse::ROOT_ID {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}
		if request.name() != fuse::NodeName::from_bytes(b"statfs.txt").unwrap()
		{
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		send_reply.ok(&resp)
	}

	fn statfs(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::StatfsRequest,
		send_reply: impl fuse::SendReply<S>,
	) -> fuse::SendResult<fuse::StatfsResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();
		let mut response = fuse::StatfsResponse::new();
		response.set_block_size(10);
		response.set_block_count(20);
		response.set_blocks_free(30);
		response.set_blocks_available(40);
		response.set_inode_count(50);
		response.set_inodes_free(60);
		response.set_max_filename_length(70);
		response.set_fragment_size(80);
		send_reply.ok(&response)
	}
}

fn statfs_test(
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
fn statfs() {
	let requests = statfs_test(|root| {
		let path = path_cstr(root.join("statfs.txt"));

		let mut stat_buf: libc::statfs = unsafe { mem::zeroed() };
		let rc = unsafe {
			libc::statfs(path.as_ptr(), (&mut stat_buf) as *mut libc::statfs)
		};
		assert_eq!(rc, 0);

		#[cfg(target_os = "linux")]
		let expect = r#"statfs {
    f_bsize: 10,
    f_blocks: 20,
    f_bfree: 30,
    f_bavail: 40,
    f_files: 50,
    f_ffree: 60,
    f_namelen: 70,
    f_frsize: 80,
}"#;
		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253424
		#[cfg(target_os = "freebsd")]
		let expect = r#"statfs {
    f_bsize: 80,
    f_iosize: 65536,
    f_blocks: 20,
    f_bfree: 30,
    f_bavail: 40,
    f_files: 50,
    f_ffree: 60,
    f_namemax: 70,
}"#;
		let got = format!("{:#?}", &DebugStatfs(stat_buf));
		if let Some(diff) = diff_str(expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let expect = r#"StatfsRequest {
    node_id: 2,
}"#;
	#[cfg(target_os = "freebsd")]
	let expect = r#"StatfsRequest {
    node_id: 1,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn statfs_statvfs() {
	let requests = statfs_test(|root| {
		let path = path_cstr(root.join("statfs.txt"));

		let mut stat_buf: libc::statvfs = unsafe { mem::zeroed() };
		let rc = unsafe {
			libc::statvfs(path.as_ptr(), (&mut stat_buf) as *mut libc::statvfs)
		};
		assert_eq!(rc, 0);

		#[cfg(target_os = "linux")]
		let expect = r#"statvfs {
    f_bsize: 10,
    f_frsize: 80,
    f_blocks: 20,
    f_bfree: 30,
    f_bavail: 40,
    f_files: 50,
    f_ffree: 60,
    f_favail: 60,
    f_namemax: 70,
}"#;
		// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=253424
		#[cfg(target_os = "freebsd")]
		let expect = r#"statvfs {
    f_bsize: 65536,
    f_frsize: 80,
    f_blocks: 20,
    f_bfree: 30,
    f_bavail: 40,
    f_files: 50,
    f_ffree: 60,
    f_favail: 60,
    f_namemax: 255,
}"#;
		let got = format!("{:#?}", &DebugStatvfs(stat_buf));
		if let Some(diff) = diff_str(expect, &got) {
			println!("{}", diff);
			assert!(false);
		}
	});
	assert_eq!(requests.len(), 1);

	#[cfg(target_os = "linux")]
	let expect = r#"StatfsRequest {
    node_id: 2,
}"#;
	#[cfg(target_os = "freebsd")]
	let expect = r#"StatfsRequest {
    node_id: 1,
}"#;
	if let Some(diff) = diff_str(expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

struct DebugStatfs(libc::statfs);

#[cfg(target_os = "linux")]
impl fmt::Debug for DebugStatfs {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("statfs")
			.field("f_bsize", &self.0.f_bsize)
			.field("f_blocks", &self.0.f_blocks)
			.field("f_bfree", &self.0.f_bfree)
			.field("f_bavail", &self.0.f_bavail)
			.field("f_files", &self.0.f_files)
			.field("f_ffree", &self.0.f_ffree)
			.field("f_namelen", &self.0.f_namelen)
			.field("f_frsize", &self.0.f_frsize)
			.finish()
	}
}

#[cfg(target_os = "freebsd")]
impl fmt::Debug for DebugStatfs {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("statfs")
			.field("f_bsize", &self.0.f_bsize)
			.field("f_iosize", &self.0.f_iosize)
			.field("f_blocks", &self.0.f_blocks)
			.field("f_bfree", &self.0.f_bfree)
			.field("f_bavail", &self.0.f_bavail)
			.field("f_files", &self.0.f_files)
			.field("f_ffree", &self.0.f_ffree)
			.field("f_namemax", &self.0.f_namemax)
			.finish()
	}
}

struct DebugStatvfs(libc::statvfs);

impl fmt::Debug for DebugStatvfs {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("statvfs")
			.field("f_bsize", &self.0.f_bsize)
			.field("f_frsize", &self.0.f_frsize)
			.field("f_blocks", &self.0.f_blocks)
			.field("f_bfree", &self.0.f_bfree)
			.field("f_bavail", &self.0.f_bavail)
			.field("f_files", &self.0.f_files)
			.field("f_ffree", &self.0.f_ffree)
			.field("f_favail", &self.0.f_favail)
			.field("f_namemax", &self.0.f_namemax)
			.finish()
	}
}
