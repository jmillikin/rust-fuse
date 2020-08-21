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

const HELLO_WORLD: &[u8] = b"Hello, world!\n";

struct HelloTxt {}

impl HelloTxt {
	fn name(&self) -> &fuse::NodeName {
		fuse::NodeName::from_bytes(b"hello.txt").unwrap()
	}

	fn node_id(&self) -> fuse::NodeId {
		fuse::NodeId::new(100).unwrap()
	}

	fn set_attr(&self, attr: &mut fuse::NodeAttr) {
		attr.set_user_id(getuid());
		attr.set_group_id(getgid());
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_size(HELLO_WORLD.len() as u64);
		attr.set_nlink(1);
	}
}

const HELLO_TXT: HelloTxt = HelloTxt {};

struct HelloWorldFS {}

impl fuse::FuseHandlers for HelloWorldFS {
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
		if request.name() != HELLO_TXT.name() {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(HELLO_TXT.node_id());
		HELLO_TXT.set_attr(node.attr_mut());

		respond.ok(&resp);
	}

	fn getattr(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::GetattrRequest,
		respond: impl for<'a> fuse::Respond<fuse::GetattrResponse<'a>>,
	) {
		let mut resp = fuse::GetattrResponse::new();
		let attr = resp.attr_mut();

		if request.node_id() == fuse::ROOT_ID {
			attr.set_user_id(getuid());
			attr.set_group_id(getgid());
			attr.set_mode(fuse::FileType::Directory | 0o755);
			attr.set_nlink(2);
			respond.ok(&resp);
			return;
		}

		if request.node_id() == HELLO_TXT.node_id() {
			HELLO_TXT.set_attr(attr);
			respond.ok(&resp);
			return;
		}

		respond.err(fuse::ErrorCode::ENOENT);
	}

	fn open(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpenRequest,
		respond: impl for<'a> fuse::Respond<fuse::OpenResponse<'a>>,
	) {
		if request.node_id() != HELLO_TXT.node_id() {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(1001);
		respond.ok(&resp);
	}

	fn read(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReadRequest,
		respond: impl for<'a> fuse::Respond<fuse::ReadResponse<'a>>,
	) {
		if request.handle() != 1001 {
			respond.err(fuse::ErrorCode::EIO);
			return;
		}

		let resp = fuse::ReadResponse::from_bytes(HELLO_WORLD);
		respond.ok(&resp);
	}

	fn opendir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpendirRequest,
		respond: impl for<'a> fuse::Respond<fuse::OpendirResponse<'a>>,
	) {
		if request.node_id() != fuse::ROOT_ID {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::OpendirResponse::new();
		resp.set_handle(1002);
		respond.ok(&resp);
	}

	fn readdir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReaddirRequest,
		respond: impl for<'a> fuse::Respond<fuse::ReaddirResponse<'a>>,
	) {
		if request.handle() != 1002 {
			respond.err(fuse::ErrorCode::EIO);
			return;
		}

		if request.cursor().is_some() {
			respond.ok(fuse::ReaddirResponse::EMPTY);
			return;
		}

		let mut resp = fuse::ReaddirResponse::with_max_size(request.size());

		let node_offset = NonZeroU64::new(1).unwrap();
		resp.add_entry(HELLO_TXT.node_id(), HELLO_TXT.name(), node_offset)
			.set_file_type(fuse::FileType::Regular);

		respond.ok(&resp);
	}

	fn releasedir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReleasedirRequest,
		respond: impl for<'a> fuse::Respond<fuse::ReleasedirResponse<'a>>,
	) {
		if request.handle() != 1002 {
			respond.err(fuse::ErrorCode::EIO);
			return;
		}

		let resp = fuse::ReleasedirResponse::new();
		respond.ok(&resp);
	}
}

fn getuid() -> u32 {
	unsafe { libc::getuid() }
}

fn getgid() -> u32 {
	unsafe { libc::getgid() }
}

fn main() {
	use fuse::os::linux;
	let mount_target = std::env::args_os().nth(1).unwrap();

	let handlers = HelloWorldFS {};
	let mut srv = linux::FuseServerBuilder::new(&mount_target, handlers)
		.set_mount(
			linux::LibcFuseMount::new()
				.set_mount_source("helloworld")
				.set_mount_subtype("helloworld"),
		)
		.build()
		.unwrap();
	srv.executor_mut().run().unwrap();
}
