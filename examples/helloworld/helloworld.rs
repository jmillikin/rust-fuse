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

use std::num::NonZeroU64;

use fuse::node;
use fuse::server::fuse_rpc;

const HELLO_WORLD: &[u8] = b"Hello, world!\n";

struct HelloTxt {}

impl HelloTxt {
	fn name(&self) -> &node::Name {
		node::Name::new("hello.txt").unwrap()
	}

	fn node_id(&self) -> node::Id {
		node::Id::new(100).unwrap()
	}

	fn set_attr(&self, attr: &mut fuse::NodeAttr) {
		attr.set_user_id(getuid());
		attr.set_group_id(getgid());
		attr.set_mode(node::Mode::S_IFREG | 0o644);
		attr.set_size(HELLO_WORLD.len() as u64);
		attr.set_nlink(1);
	}
}

const HELLO_TXT: HelloTxt = HelloTxt {};

struct HelloWorldFS {}

impl<S: fuse_rpc::FuseSocket> fuse_rpc::FuseHandlers<S> for HelloWorldFS {
	fn lookup(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::LookupRequest,
	) -> fuse_rpc::FuseResult<fuse::LookupResponse, S::Error> {
		if !request.parent_id().is_root() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}
		if request.name() != HELLO_TXT.name() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(HELLO_TXT.node_id());
		HELLO_TXT.set_attr(node.attr_mut());

		call.respond_ok(&resp)
	}

	fn getattr(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::GetattrRequest,
	) -> fuse_rpc::FuseResult<fuse::GetattrResponse, S::Error> {
		let mut resp = fuse::GetattrResponse::new();
		let attr = resp.attr_mut();

		if request.node_id().is_root() {
			attr.set_user_id(getuid());
			attr.set_group_id(getgid());
			attr.set_mode(node::Mode::S_IFDIR | 0o755);
			attr.set_nlink(2);
			return call.respond_ok(&resp);
		}

		if request.node_id() == HELLO_TXT.node_id() {
			HELLO_TXT.set_attr(attr);
			return call.respond_ok(&resp);
		}

		call.respond_err(fuse::Error::NOT_FOUND)
	}

	fn open(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::OpenRequest,
	) -> fuse_rpc::FuseResult<fuse::OpenResponse, S::Error> {
		if request.node_id() != HELLO_TXT.node_id() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}

		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(1001);
		call.respond_ok(&resp)
	}

	fn read(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::ReadRequest,
	) -> fuse_rpc::FuseResult<fuse::ReadResponse, S::Error> {
		if request.handle() != 1001 {
			return call.respond_err(fuse::Error::INVALID_ARGUMENT);
		}

		let resp = fuse::ReadResponse::from_bytes(HELLO_WORLD);
		call.respond_ok(&resp)
	}

	fn opendir(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::OpendirRequest,
	) -> fuse_rpc::FuseResult<fuse::OpendirResponse, S::Error> {
		if !request.node_id().is_root() {
			return call.respond_err(fuse::Error::NOT_FOUND);
		}

		let mut resp = fuse::OpendirResponse::new();
		resp.set_handle(1002);
		call.respond_ok(&resp)
	}

	fn readdir(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::ReaddirRequest,
	) -> fuse_rpc::FuseResult<fuse::ReaddirResponse, S::Error> {
		if request.handle() != 1002 {
			return call.respond_err(fuse::Error::INVALID_ARGUMENT);
		}

		if request.offset().is_some() {
			return call.respond_ok(fuse::ReaddirResponse::EMPTY);
		}

		let mut buf = vec![0u8; request.size()];
		let mut entries = fuse::ReaddirEntriesWriter::new(&mut buf);

		let node_offset = NonZeroU64::new(1).unwrap();
		let mut entry = fuse::ReaddirEntry::new(
			HELLO_TXT.node_id(),
			HELLO_TXT.name(),
			node_offset,
		);
		entry.set_file_type(node::Type::Regular);
		entries.try_push(&entry).unwrap();

		let resp = fuse::ReaddirResponse::new(entries.into_entries());
		call.respond_ok(&resp)
	}

	fn releasedir(
		&self,
		call: fuse_rpc::FuseCall<S>,
		request: &fuse::ReleasedirRequest,
	) -> fuse_rpc::FuseResult<fuse::ReleasedirResponse, S::Error> {
		if request.handle() != 1002 {
			return call.respond_err(fuse::Error::INVALID_ARGUMENT);
		}

		let resp = fuse::ReleasedirResponse::new();
		call.respond_ok(&resp)
	}
}

fn getuid() -> u32 {
	unsafe { libc::getuid() }
}

fn getgid() -> u32 {
	unsafe { libc::getgid() }
}

fn main() {
	use std::ffi::CString;
	use std::os::unix::ffi::OsStrExt;

	let mount_target = std::env::args_os().nth(1).unwrap();
	let mount_target_cstr = CString::new(mount_target.as_bytes()).unwrap();

	let handlers = HelloWorldFS {};

	let fs_source = CString::new("helloworld").unwrap();
	let fs_subtype = CString::new("helloworld").unwrap();

	let dev_fuse;

	#[cfg(target_os = "linux")]
	{
		use fuse_libc::os::linux as fuse_libc;

		let mut mount_options = fuse::os::linux::MountOptions::new();
		mount_options.set_source(Some(&fs_source));
		mount_options.set_fs_subtype(Some(&fs_subtype));
		mount_options.set_user_id(Some(getuid()));
		mount_options.set_group_id(Some(getgid()));
		dev_fuse = fuse_libc::mount(&mount_target_cstr, mount_options)
			.unwrap();
	}

	#[cfg(target_os = "freebsd")]
	{
		use fuse_libc::os::freebsd as fuse_libc;

		let mut mount_options = fuse::os::freebsd::MountOptions::new();
		mount_options.set_fs_subtype(Some(&fs_subtype));
		dev_fuse = fuse_libc::mount(&mount_target_cstr, mount_options)
			.unwrap();
	}

	let srv = fuse_rpc::FuseServerBuilder::new(dev_fuse, handlers)
		.fuse_init()
		.unwrap();

	srv.serve().unwrap();
}
