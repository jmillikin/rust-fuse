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

use fuse::server;
use fuse::server::FuseRequest;

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	path_cstr,
	OsError,
};

struct TestFS {
	block_size: u32,
	requests: mpsc::Sender<String>,
}

struct TestHandlers<'a, S> {
	fs: &'a TestFS,
	conn: &'a server::FuseConnection<S>,
}

const FIBMAP: i32 = 1; // IO(0x00,1)

impl interop_testutil::TestFS for TestFS {
	fn dispatch_request(
		&self,
		conn: &server::FuseConnection<interop_testutil::DevFuse>,
		request: FuseRequest<'_>,
	) {
		use fuse::server::FuseHandlers;
		(TestHandlers{fs: self, conn}).dispatch(request);
	}

	fn mount_type(&self) -> &'static fuse::os::linux::MountType {
		fuse::os::linux::MountType::FUSEBLK
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

impl<'a, S> server::FuseHandlers for TestHandlers<'a, S>
where
	S: server::FuseSocket,
	S::Error: core::fmt::Debug,
{
	fn unimplemented(&self, request: FuseRequest<'_>) {
		self.conn.reply(request.id()).err(OsError::UNIMPLEMENTED).unwrap();
	}

	fn bmap(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::BmapRequest::try_from(request).unwrap();

		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		let mut reply = fuse::kernel::fuse_bmap_out::new();
		reply.block = 5678;
		send_reply.ok(&reply).unwrap();
	}

	fn lookup(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::LookupRequest::try_from(request).unwrap();

		if !request.parent_id().is_root() {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}
		if request.name() != "file.txt" {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_link_count(1);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn open(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		if request.header().raw().nodeid != 2 {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}
		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 10;
		send_reply.ok(&reply).unwrap();
	}

	fn release(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		send_reply.ok_empty().unwrap();
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
