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

use std::ffi::CString;
use std::num::NonZeroU16;
use std::thread;

const HELLO_WORLD: &[u8] = b"Hello, world!\n";

struct HelloTxt {}

impl HelloTxt {
	fn name(&self) -> &[u8] {
		b"hello.txt"
	}

	fn node_id(&self) -> fuse::NodeId {
		fuse::NodeId::new(100).unwrap()
	}

	fn set_attr(&self, attr: &mut fuse::NodeAttr) {
		attr.set_user_id(getuid());
		attr.set_group_id(getgid());
		attr.set_mode(libc::S_IFREG | 0o644);
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
		respond: impl for<'a> fuse::RespondOnce<fuse::LookupResponse<'a>>,
	) {
		if request.parent_id() != fuse::NodeId::ROOT {
			respond.err(err_not_found());
			return;
		}
		if request.name().as_bytes() != HELLO_TXT.name() {
			respond.err(err_not_found());
			return;
		}

		let respond = respond.into_box();
		thread::spawn(move || {
			let mut resp = fuse::LookupResponse::new();
			resp.set_node_id(HELLO_TXT.node_id());
			HELLO_TXT.set_attr(resp.attr_mut());

			respond.ok(&resp);
		});
	}

	fn getattr(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::GetattrRequest,
		respond: impl for<'a> fuse::RespondOnce<fuse::GetattrResponse<'a>>,
	) {
		let node_id = request.node_id();

		let respond = respond.into_box();
		thread::spawn(move || {
			let mut resp = fuse::GetattrResponse::new();
			let attr = resp.attr_mut();

			if node_id == fuse::NodeId::ROOT {
				attr.set_user_id(getuid());
				attr.set_group_id(getgid());
				attr.set_mode(libc::S_IFDIR | 0o755);
				attr.set_nlink(2);
				respond.ok(&resp);
				return;
			}

			if node_id == HELLO_TXT.node_id() {
				HELLO_TXT.set_attr(attr);
				respond.ok(&resp);
				return;
			}

			respond.err(err_not_found());
		});
	}

	fn open(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpenRequest,
		respond: impl for<'a> fuse::RespondOnce<fuse::OpenResponse<'a>>,
	) {
		if request.node_id() != HELLO_TXT.node_id() {
			respond.err(err_not_found());
			return;
		}

		let respond = respond.into_box();
		thread::spawn(move || {
			let mut resp = fuse::OpenResponse::new();
			resp.set_handle(1001);
			respond.ok(&resp);
		});
	}

	fn read(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReadRequest,
		respond: impl for<'a> fuse::RespondOnce<fuse::ReadResponse<'a>>,
	) {
		if request.handle() != 1001 {
			respond.err(err_io());
			return;
		}

		let respond = respond.into_box();
		thread::spawn(move || {
			let resp = fuse::ReadResponse::from_bytes(HELLO_WORLD);
			respond.ok(&resp);
		});
	}

	fn opendir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpendirRequest,
		respond: impl for<'a> fuse::RespondOnce<fuse::OpendirResponse<'a>>,
	) {
		if request.node_id() != fuse::NodeId::ROOT {
			respond.err(err_not_found());
			return;
		}

		let respond = respond.into_box();
		thread::spawn(move || {
			let mut resp = fuse::OpendirResponse::new();
			resp.set_handle(1002);
			respond.ok(&resp);
		});
	}

	fn readdir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReaddirRequest,
		respond: impl for<'a> fuse::RespondOnce<fuse::ReaddirResponse<'a>>,
	) {
		if request.handle() != 1002 {
			respond.err(err_io());
			return;
		}

		// TODO: fix ReaddirResponse::new
		let mut resp = fuse::ReaddirResponse::new(request);
		if request.offset() != 0 {
			respond.ok(&resp);
			return;
		}

		{
			let name = CString::new("hello.txt").unwrap();
			let mut dirent = resp.push(HELLO_TXT.node_id(), 1, &name).unwrap();
			dirent.set_node_kind(fuse::NodeKind::REG);
			if let Some(node) = dirent.node_mut() {
				HELLO_TXT.set_attr(node.attr_mut());
			}
		}

		respond.ok(&resp);
	}

	fn releasedir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReleasedirRequest,
		respond: impl for<'a> fuse::RespondOnce<fuse::ReleasedirResponse<'a>>,
	) {
		if request.handle() != 1002 {
			respond.err(err_io());
			return;
		}

		let respond = respond.into_box();
		thread::spawn(move || {
			let resp = fuse::ReleasedirResponse::new();
			respond.ok(&resp);
		});
	}
}

fn err_not_found() -> NonZeroU16 {
	unsafe { NonZeroU16::new_unchecked(libc::ENOENT as u16) }
}

fn err_io() -> NonZeroU16 {
	unsafe { NonZeroU16::new_unchecked(libc::EIO as u16) }
}

fn getuid() -> u32 {
	unsafe { libc::getuid() }
}

fn getgid() -> u32 {
	unsafe { libc::getgid() }
}

fn main() {
	let mount_target = std::env::args_os().nth(1).unwrap();

	let srv = fuse::FuseServerBuilder::new(HelloWorldFS {})
		.set_mount_options(
			fuse::os::linux::FuseMountOptions::new()
				.set_mount_source("helloworld")
				.set_mount_subtype("helloworld"),
		)
		.mount(mount_target)
		.unwrap();

	let executor_arc = srv.executor().clone();
	let mut executor = executor_arc.lock().unwrap();

	executor.run().unwrap();
}
