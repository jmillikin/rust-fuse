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

use fuse::server::fuse_rpc;
use fuse::server::prelude::*;

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

struct HelloWorldFS {}

impl<S: FuseSocket> fuse_rpc::Handlers<S> for HelloWorldFS {
	fn lookup(
		&self,
		call: fuse_rpc::Call<S>,
		request: &LookupRequest,
	) -> fuse_rpc::SendResult<LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}
		if request.name() != HELLO_TXT.name() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}

		let mut attr = fuse::Attributes::new(HELLO_TXT.node_id());
		HELLO_TXT.set_attr(&mut attr);

		let entry = fuse::Entry::new(attr);
		let resp = LookupResponse::new(Some(entry));
		call.respond_ok(&resp)
	}

	fn getattr(
		&self,
		call: fuse_rpc::Call<S>,
		request: &GetattrRequest,
	) -> fuse_rpc::SendResult<GetattrResponse, S::Error> {
		let mut attr = fuse::Attributes::new(request.node_id());

		if request.node_id().is_root() {
			attr.set_user_id(getuid());
			attr.set_group_id(getgid());
			attr.set_mode(fuse::FileMode::S_IFDIR | 0o755);
			attr.set_link_count(2);
			let resp = GetattrResponse::new(attr);
			return call.respond_ok(&resp);
		}

		if request.node_id() == HELLO_TXT.node_id() {
			HELLO_TXT.set_attr(&mut attr);
			let resp = GetattrResponse::new(attr);
			return call.respond_ok(&resp);
		}

		call.respond_err(fuse::Error::NOT_FOUND)
	}

	fn open(
		&self,
		call: fuse_rpc::Call<S>,
		request: &OpenRequest,
	) -> fuse_rpc::SendResult<OpenResponse, S::Error> {
		if request.node_id() != HELLO_TXT.node_id() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}

		let mut resp = OpenResponse::new();
		resp.set_handle(1001);
		call.respond_ok(&resp)
	}

	fn read(
		&self,
		call: fuse_rpc::Call<S>,
		request: &ReadRequest,
	) -> fuse_rpc::SendResult<ReadResponse, S::Error> {
		if request.handle() != 1001 {
			return call.respond_err(fuse::Error::INVALID_ARGUMENT);
		}

		let resp = ReadResponse::from_bytes(HELLO_WORLD);
		call.respond_ok(&resp)
	}

	fn opendir(
		&self,
		call: fuse_rpc::Call<S>,
		request: &OpendirRequest,
	) -> fuse_rpc::SendResult<OpendirResponse, S::Error> {
		if !request.node_id().is_root() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}

		let mut resp = OpendirResponse::new();
		resp.set_handle(1002);
		call.respond_ok(&resp)
	}

	fn readdir(
		&self,
		call: fuse_rpc::Call<S>,
		request: &ReaddirRequest,
	) -> fuse_rpc::SendResult<ReaddirResponse, S::Error> {
		if request.handle() != 1002 {
			return call.respond_err(fuse::Error::INVALID_ARGUMENT);
		}

		if request.offset().is_some() {
			return call.respond_ok(ReaddirResponse::EMPTY);
		}

		let mut buf = vec![0u8; request.size()];
		let mut entries = ReaddirEntriesWriter::new(&mut buf);

		let node_offset = NonZeroU64::new(1).unwrap();
		let mut entry = ReaddirEntry::new(
			HELLO_TXT.node_id(),
			HELLO_TXT.name(),
			node_offset,
		);
		entry.set_file_type(fuse::FileType::Regular);
		entries.try_push(&entry).unwrap();

		let resp = ReaddirResponse::new(entries.into_entries());
		call.respond_ok(&resp)
	}

	fn releasedir(
		&self,
		call: fuse_rpc::Call<S>,
		request: &ReleasedirRequest,
	) -> fuse_rpc::SendResult<ReleasedirResponse, S::Error> {
		if request.handle() != 1002 {
			return call.respond_err(fuse::Error::INVALID_ARGUMENT);
		}

		let resp = ReleasedirResponse::new();
		call.respond_ok(&resp)
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
	let handlers = HelloWorldFS {};
	let mount_target = std::env::args_os().nth(1).unwrap();
	let dev_fuse = mount(&mount_target);
	let conn = FuseServer::new().connect(dev_fuse).unwrap();
	fuse_std::serve_fuse(&conn, &handlers);
}
