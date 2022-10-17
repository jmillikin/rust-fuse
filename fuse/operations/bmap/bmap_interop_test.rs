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

use std::ffi;
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
	block_size: u32,
	requests: mpsc::Sender<String>,
}

const FIBMAP: i32 = 1; // IO(0x00,1)

impl interop_testutil::TestFS for TestFS {
	fn mount_fs_type(&self) -> ffi::CString {
		ffi::CString::new("fuseblk").unwrap()
	}

	fn mount_source(&self) -> ffi::CString {
		ffi::CString::new("/dev/loop0").unwrap()
	}

	fn linux_mount_options(
		&self,
		mount_options: &mut fuse::os::linux::MountOptions,
	) {
		mount_options.set_block_size(Some(self.block_size));
	}
}

impl<S: fuse_rpc::FuseSocket> fuse_rpc::Handlers<S> for TestFS {
	fn bmap(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::BmapRequest,
	) -> fuse_rpc::SendResult<fuse::BmapResponse, S::Error> {
		self.requests.send(format!("{:#?}", request)).unwrap();

		let mut resp = fuse::BmapResponse::new();
		resp.set_block(5678);
		call.respond_ok(&resp)
	}

	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &fuse::LookupRequest,
	) -> fuse_rpc::SendResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(ErrorCode::ENOENT);
		}
		if request.name() != "file.txt" {
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
		request: &fuse::OpenRequest,
	) -> fuse_rpc::SendResult<fuse::OpenResponse, S::Error> {
		if request.node_id().get() != 2 {
			return call.respond_err(ErrorCode::ENOENT);
		}
		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(10);
		call.respond_ok(&resp)
	}

	fn release(
		&self,
		call: fuse_rpc::Call<S>,
		_request: &fuse::ReleaseRequest,
	) -> fuse_rpc::SendResult<fuse::ReleaseResponse, S::Error> {
		let resp = fuse::ReleaseResponse::new();
		call.respond_ok(&resp)
	}
}

fn bmap_test(
	block_size: u32,
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		block_size,
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

unsafe fn ioctl_fibmap(fd: i32, block: usize) -> usize {
	let mut ioctl_arg = block;

	let rc = libc::ioctl(fd, FIBMAP, &mut ioctl_arg as *mut usize);
	assert_eq!(rc, 0);

	ioctl_arg
}

#[test]
fn test_bmap() {
	let requests = bmap_test(512, |root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe {
			libc::open(path.as_ptr(), libc::O_RDONLY)
		};
		assert_ne!(fd, -1);

		let mapped_block = unsafe { ioctl_fibmap(fd, 1234) };

		assert_eq!(mapped_block, 5678);

		unsafe { libc::close(fd); }
	});
	assert_eq!(requests.len(), 1);

	let expect = format!(
		r#"BmapRequest {{
    node_id: 2,
    block: 1234,
    block_size: 512,
}}"#,
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}

#[test]
fn test_bmap_larger_block_size() {
	let requests = bmap_test(1024, |root| {
		let path = path_cstr(root.join("file.txt"));

		let fd = unsafe {
			libc::open(path.as_ptr(), libc::O_RDONLY)
		};
		assert_ne!(fd, -1);

		let mapped_block = unsafe { ioctl_fibmap(fd, 1234) };

		assert_eq!(mapped_block, 5678);

		unsafe { libc::close(fd); }
	});
	assert_eq!(requests.len(), 1);

	let expect = format!(
		r#"BmapRequest {{
    node_id: 2,
    block: 1234,
    block_size: 1024,
}}"#,
	);
	if let Some(diff) = diff_str(&expect, &requests[0]) {
		println!("{}", diff);
		assert!(false);
	}
}
