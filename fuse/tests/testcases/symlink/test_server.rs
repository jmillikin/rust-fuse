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
		if request.name() != fuse::NodeName::from_bytes(b"exists.txt").unwrap()
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

	fn symlink(
		&self,
		_ctx: fuse::ServerContext,
		request: &fuse::SymlinkRequest,
		respond: impl for<'a> fuse::Respond<fuse::SymlinkResponse<'a>>,
	) {
		println!("\n{:#?}", request);

		let mut resp = fuse::SymlinkResponse::new();
		let node = resp.node_mut();
		node.set_id(fuse::NodeId::new(3).unwrap());

		let attr = node.attr_mut();
		attr.set_mode(fuse::FileType::Symlink | 0o644);
		attr.set_nlink(1);

		println!("{:#?}", resp);
		respond.ok(&resp);
	}
}

fn main() {
	test_server_base::main(TestFS {})
}
