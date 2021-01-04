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

struct PrintHooks {}

impl fuse::ServerHooks for PrintHooks {
	fn unknown_request(&self, request: &fuse::UnknownRequest) {
		println!("\n[unknown_request]\n{:#?}", request);
	}

	fn unhandled_request(&self, request_header: &fuse::RequestHeader) {
		println!("\n[unhandled_request]\n{:#?}", request_header);
	}

	fn request_error(
		&self,
		request_header: &fuse::RequestHeader,
		err: fuse::Error,
	) {
		println!("\n[request_error]\n{:#?}", request_header);
		println!("{:#?}", err);
	}

	fn response_error(
		&self,
		request_header: &fuse::RequestHeader,
		code: Option<fuse::ErrorCode>,
	) {
		println!("\n[response_error]\n{:#?}", request_header);
		println!("{:#?}", code);
	}

	fn async_channel_error(
		&self,
		request_header: &fuse::RequestHeader,
		code: Option<fuse::ErrorCode>,
	) {
		println!("\n[async_channel_error]\n{:#?}", request_header);
		println!("{:#?}", code);
	}
}

pub fn main(handlers: impl fuse::FuseHandlers) {
	use fuse::os::linux;
	println!("START {}", std::env::args().next().unwrap());

	let mut srv = linux::FuseServerBuilder::new("/rust-fuse/testfs", handlers)
		.set_mount(
			linux::SyscallFuseMount::new()
				.set_mount_source("testfs")
				.set_mount_subtype("testfs"),
		)
		.set_hooks(PrintHooks {})
		.build()
		.unwrap();
	srv.executor_mut().run().unwrap();
}
