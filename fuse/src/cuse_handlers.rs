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

use crate::error::ErrorCode;
use crate::protocol;
use crate::server;

/// User-provided handlers for CUSE operations.
pub trait CuseHandlers {
	fn cuse_init(
		&mut self,
		request: &protocol::CuseInitRequest,
	) -> protocol::CuseInitResponse {
		protocol::CuseInitResponse::for_request_impl(request)
	}

	#[cfg(any(doc, feature = "unstable_flush"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_flush")))]
	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		respond: impl for<'a> server::Respond<protocol::FlushResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fsync"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fsync")))]
	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		respond: impl for<'a> server::Respond<protocol::FsyncResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_ioctl")))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		respond: impl for<'a> server::Respond<protocol::IoctlResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		respond: impl for<'a> server::Respond<protocol::OpenResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		respond: impl for<'a> server::Respond<protocol::ReadResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		respond: impl for<'a> server::Respond<protocol::ReleaseResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		respond: impl for<'a> server::Respond<protocol::WriteResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}
}
