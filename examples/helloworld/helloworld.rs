// Copyright 2020 John Millikin and the rust-fuse contributors.
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

use std::ffi::{CString, OsStr};
use std::num::NonZeroU64;
use std::os::unix::ffi::OsStrExt;

use fuse::server;
use fuse::server::FuseRequest;

#[cfg(target_os = "freebsd")]
pub use fuse::os::freebsd::OsError;

#[cfg(target_os = "linux")]
pub use fuse::os::linux::OsError;

const HELLO_WORLD: &[u8] = b"Hello, world!\n";

struct HelloTxt {}

impl HelloTxt {
	fn name(&self) -> &fuse::NodeName {
		fuse::NodeName::new("hello.txt").unwrap()
	}

	fn node_id(&self) -> fuse::NodeId {
		fuse::NodeId::new(100).unwrap()
	}

	fn set_attr(&self, attr: &mut fuse::Attributes) {
		attr.set_user_id(getuid());
		attr.set_group_id(getgid());
		attr.set_mode(fuse::FileMode::S_IFREG | 0o644);
		attr.set_size(HELLO_WORLD.len() as u64);
		attr.set_link_count(1);
	}
}

const HELLO_TXT: HelloTxt = HelloTxt {};

struct HelloWorldFS<'a, S> {
	conn: &'a server::FuseConnection<S>,
}

impl<S> server::FuseHandlers for HelloWorldFS<'_, S>
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
			send_reply.err(OsError::NOT_FOUND).unwrap();
			return;
		}
		if request.name() != HELLO_TXT.name() {
			send_reply.err(OsError::NOT_FOUND).unwrap();
			return;
		}

		let mut attr = fuse::Attributes::new(HELLO_TXT.node_id());
		HELLO_TXT.set_attr(&mut attr);

		send_reply.ok(&fuse::Entry::new(attr)).unwrap();
	}

	fn getattr(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::GetattrRequest::try_from(request).unwrap();

		let mut attr = fuse::Attributes::new(request.node_id());
		if request.node_id().is_root() {
			attr.set_user_id(getuid());
			attr.set_group_id(getgid());
			attr.set_mode(fuse::FileMode::S_IFDIR | 0o755);
			attr.set_link_count(2);

			let mut reply = fuse::kernel::fuse_attr_out::new();
			reply.attr = *attr.raw();
			send_reply.ok(&reply).unwrap();
			return;
		}

		if request.node_id() == HELLO_TXT.node_id() {
			HELLO_TXT.set_attr(&mut attr);
			let mut reply = fuse::kernel::fuse_attr_out::new();
			reply.attr = *attr.raw();
			send_reply.ok(&reply).unwrap();
			return;
		}

		send_reply.err(OsError::NOT_FOUND).unwrap();
	}

	fn open(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::OpenRequest::try_from(request).unwrap();
		if request.node_id() != HELLO_TXT.node_id() {
			send_reply.err(OsError::NOT_FOUND).unwrap();
			return;
		}
		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 1001;
		send_reply.ok(&reply).unwrap();
	}

	fn read(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReadRequest::try_from(request).unwrap();
		if request.handle() != 1001 {
			send_reply.err(OsError::INVALID_ARGUMENT).unwrap();
			return;
		}
		send_reply.ok_buf(HELLO_WORLD).unwrap();
	}

	fn opendir(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::OpendirRequest::try_from(request).unwrap();
		if !request.node_id().is_root() {
			send_reply.err(OsError::NOT_FOUND).unwrap();
			return;
		}
		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 1002;
		send_reply.ok(&reply).unwrap();
	}

	fn readdir(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReaddirRequest::try_from(request).unwrap();

		if request.handle() != 1002 {
			send_reply.err(OsError::INVALID_ARGUMENT).unwrap();
			return;
		}

		if request.offset().is_some() {
			send_reply.ok_empty().unwrap();
			return;
		}

		let mut buf = vec![0u8; request.size()];
		let mut entries = server::ReaddirEntriesWriter::new(&mut buf);

		let node_offset = NonZeroU64::new(1).unwrap();
		let mut entry = server::ReaddirEntry::new(
			HELLO_TXT.node_id(),
			HELLO_TXT.name(),
			node_offset,
		);
		entry.set_file_type(fuse::FileType::Regular);
		entries.try_push(&entry).unwrap();

		send_reply.ok(&entries.into_entries()).unwrap();
	}

	fn releasedir(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReleasedirRequest::try_from(request).unwrap();
		if request.handle() != 1002 {
			send_reply.err(OsError::INVALID_ARGUMENT).unwrap();
			return;
		}
		send_reply.ok_empty().unwrap();
	}
}

fn getuid() -> u32 {
	unsafe { libc::getuid() }
}

fn getgid() -> u32 {
	unsafe { libc::getgid() }
}

#[cfg(target_os = "linux")]
fn mount(target: &OsStr) -> fuse_libc::FuseServerSocket {
	use fuse::os::linux::FuseSubtype;
	use fuse::os::linux::MountSource;

	let target_cstr = CString::new(target.as_bytes()).unwrap();
	let fs_source = MountSource::new(c"helloworld").unwrap();
	let fs_subtype = FuseSubtype::new(c"helloworld").unwrap();

	let mut mount_options = fuse::os::linux::MountOptions::new();
	mount_options.set_mount_source(fs_source);
	mount_options.set_subtype(Some(fs_subtype));
	mount_options.set_user_id(Some(getuid()));
	mount_options.set_group_id(Some(getgid()));
	fuse_libc::os::linux::mount(&target_cstr, mount_options).unwrap()
}

#[cfg(target_os = "freebsd")]
fn mount(target: &OsStr) -> fuse_libc::FuseServerSocket {
	use fuse::os::freebsd::FuseSubtype;

	let target_cstr = CString::new(target.as_bytes()).unwrap();
	let fs_subtype = FuseSubtype::new(c"helloworld").unwrap();

	let mut mount_options = fuse::os::freebsd::MountOptions::new();
	mount_options.set_subtype(Some(fs_subtype));
	fuse_libc::os::freebsd::mount(&target_cstr, mount_options).unwrap()
}

fn main() {
	let mount_target = std::env::args_os().nth(1).unwrap();
	let dev_fuse = mount(&mount_target);
	let conn = server::FuseServer::new().connect(dev_fuse).unwrap();
	let handlers = HelloWorldFS {
		conn: &conn,
	};
	fuse_std::serve_fuse(&conn, &handlers);
}
