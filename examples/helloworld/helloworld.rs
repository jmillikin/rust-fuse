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

use fuse::server::basic;

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

impl<S: fuse::io::OutputStream> basic::FuseHandlers<S> for HelloWorldFS {
	fn lookup(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::LookupRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::LookupResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		if request.parent_id() != fuse::ROOT_ID {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}
		if request.name() != HELLO_TXT.name() {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(HELLO_TXT.node_id());
		HELLO_TXT.set_attr(node.attr_mut());

		send_reply.ok(&resp)
	}

	fn getattr(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::GetattrRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::GetattrResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		let mut resp = fuse::GetattrResponse::new();
		let attr = resp.attr_mut();

		if request.node_id() == fuse::ROOT_ID {
			attr.set_user_id(getuid());
			attr.set_group_id(getgid());
			attr.set_mode(fuse::FileType::Directory | 0o755);
			attr.set_nlink(2);
			return send_reply.ok(&resp);
		}

		if request.node_id() == HELLO_TXT.node_id() {
			HELLO_TXT.set_attr(attr);
			return send_reply.ok(&resp);
		}

		send_reply.err(fuse::ErrorCode::ENOENT)
	}

	fn open(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::OpenRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::OpenResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		if request.node_id() != HELLO_TXT.node_id() {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let mut resp = fuse::OpenResponse::new();
		resp.set_handle(1001);
		send_reply.ok(&resp)
	}

	fn read(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::ReadRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::ReadResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		if request.handle() != 1001 {
			return send_reply.err(fuse::ErrorCode::EIO);
		}

		let resp = fuse::ReadResponse::from_bytes(HELLO_WORLD);
		send_reply.ok(&resp)
	}

	fn opendir(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::OpendirRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::OpendirResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		if request.node_id() != fuse::ROOT_ID {
			return send_reply.err(fuse::ErrorCode::ENOENT);
		}

		let mut resp = fuse::OpendirResponse::new();
		resp.set_handle(1002);
		send_reply.ok(&resp)
	}

	fn readdir(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::ReaddirRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::ReaddirResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		if request.handle() != 1002 {
			return send_reply.err(fuse::ErrorCode::EIO);
		}

		if request.cursor().is_some() {
			return send_reply.ok(fuse::ReaddirResponse::EMPTY);
		}

		let mut resp = fuse::ReaddirResponse::with_max_size(request.size());

		let node_offset = NonZeroU64::new(1).unwrap();
		resp.add_entry(HELLO_TXT.node_id(), HELLO_TXT.name(), node_offset)
			.set_file_type(fuse::FileType::Regular);

		send_reply.ok(&resp)
	}

	fn releasedir(
		&self,
		_ctx: basic::ServerContext,
		request: &fuse::ReleasedirRequest,
		send_reply: impl for<'a> basic::SendReply<S, fuse::ReleasedirResponse<'a>>,
	) -> Result<(), fuse::io::Error<S::Error>> {
		if request.handle() != 1002 {
			return send_reply.err(fuse::ErrorCode::EIO);
		}

		let resp = fuse::ReleasedirResponse::new();
		send_reply.ok(&resp)
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

	let dev_fuse = linux::LibcFuseMount::new()
		.set_mount_source("helloworld")
		.set_mount_subtype("helloworld")
		.mount(mount_target.as_ref())
		.unwrap();
	let conn = fuse::server::FuseConnectionBuilder::new(dev_fuse)
		.build()
		.unwrap();
	let srv = basic::FuseServerBuilder::new(conn, handlers).build();

	let mut buf = fuse::io::ArrayBuffer::new();
	srv.serve(&mut buf).unwrap();
}
