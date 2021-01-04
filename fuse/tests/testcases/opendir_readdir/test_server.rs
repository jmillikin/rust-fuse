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
		if request.name() != fuse::NodeName::from_bytes(b"readdir.d").unwrap() {
			respond.err(fuse::ErrorCode::ENOENT);
			return;
		}

		let mut resp = fuse::LookupResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(2).unwrap());
		node.set_cache_timeout(std::time::Duration::from_secs(60));

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Directory | 0o755);
		attr.set_nlink(2);

		println!("{:#?}", resp);
		respond.ok(&resp);
	}

	fn opendir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::OpendirRequest,
		respond: impl for<'a> fuse::Respond<fuse::OpendirResponse<'a>>,
	) {
		println!("\n{:#?}", request);

		let mut resp = fuse::OpendirResponse::new();
		resp.set_handle(12345);

		println!("{:#?}", resp);
		respond.ok(&resp);
	}

	fn readdir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReaddirRequest,
		respond: impl for<'a> fuse::Respond<fuse::ReaddirResponse<'a>>,
	) {
		println!("\n{:#?}", request);

		let mut cursor: u64 = match request.cursor() {
			Some(x) => x.into(),
			None => 0,
		};

		let mut resp = fuse::ReaddirResponse::with_max_size(request.size());
		if cursor == 0 {
			cursor += 1;
			let entry = resp.add_entry(
				fuse::NodeId::new(10).unwrap(),
				fuse::NodeName::from_bytes(b"entry_a").unwrap(),
				NonZeroU64::new(cursor).unwrap(),
			);
			entry.set_file_type(fuse::FileType::Regular);
		}
		if cursor == 1 {
			cursor += 1;
			let entry = resp.add_entry(
				fuse::NodeId::new(11).unwrap(),
				fuse::NodeName::from_bytes(b"entry_b").unwrap(),
				NonZeroU64::new(cursor).unwrap(),
			);
			entry.set_file_type(fuse::FileType::Symlink);

			println!("{:#?}", resp);
			respond.ok(&resp);
			return;
		}

		if cursor == 2 {
			cursor += 1;
			let entry = resp.add_entry(
				fuse::NodeId::new(12).unwrap(),
				fuse::NodeName::from_bytes(b"entry_c").unwrap(),
				NonZeroU64::new(cursor).unwrap(),
			);
			entry.set_file_type(fuse::FileType::Directory);
		}

		println!("{:#?}", resp);
		respond.ok(&resp);
	}

	fn releasedir(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::ReleasedirRequest,
		respond: impl for<'a> fuse::Respond<fuse::ReleasedirResponse<'a>>,
	) {
		println!("\n{:#?}", request);

		let resp = fuse::ReleasedirResponse::new();

		println!("{:#?}", resp);
		respond.ok(&resp);
	}
}

fn main() {
	test_server_base::main(TestFS {})
}
