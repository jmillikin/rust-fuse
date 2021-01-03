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

use std::num::NonZeroU64;

struct TestFS {}

impl fuse::FuseHandlers for TestFS {
	fn lookup(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::LookupRequest,
		respond: impl for<'a> fuse::Respond<fuse::LookupResponse<'a>>,
	) {
		println!("\n{:#?}", request);
		if request.parent_id() != fuse::ROOT_ID {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}
		if request.name() != fuse::NodeName::from_bytes(b"xattrs.txt").unwrap()
		{
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Regular | 0o644);
		attr.set_nlink(1);

		println!("{:#?}", resp);
		respond.ok(&resp);
	}

	fn getxattr(
		&self,
		ctx: fuse::ServerContext,
		request: &fuse::GetxattrRequest,
		respond: impl for<'a> fuse::Respond<fuse::GetxattrResponse<'a>>,
	) {
		println!("\n{:#?}", request);

		let xattr_small = fuse::XattrName::from_bytes(b"xattr_small").unwrap();
		let xattr_toobig =
			fuse::XattrName::from_bytes(b"xattr_toobig").unwrap();

		if request.name() == xattr_small {
			let mut resp = fuse::GetxattrResponse::new(request.size());
			match resp.try_set_value(b"small xattr value") {
				Ok(_) => {
					println!("{:#?}", resp);
					respond.ok(&resp);
				},
				Err(_) => {
					// TODO: error should either have enough public info to let the caller
					// return an appropriate error code, or ERANGE should be handled by
					// the response dispatcher.
					respond.err(fuse::ErrorCode::ERANGE);
				},
			}
			return;
		}

		if request.name() == xattr_toobig {
			respond.err(fuse::ErrorCode::E2BIG);
			return;
		}

		respond.err(fuse::ErrorCode::ENOATTR);
	}
}

fn main() {
	use fuse::os::linux;
	println!("START {}", std::env::args().next().unwrap());

	let handlers = TestFS {};
	let mut srv = linux::FuseServerBuilder::new("/rust-fuse/testfs", handlers)
		.set_mount(
			linux::SyscallFuseMount::new()
				.set_mount_source("testfs")
				.set_mount_subtype("testfs"),
		)
		.build()
		.unwrap();
	srv.executor_mut().run().unwrap();
}
