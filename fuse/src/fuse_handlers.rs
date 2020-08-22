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

/// User-provided handlers for FUSE operations.
///
/// Most FUSE handlers (with the exception of [`fuse_init`]) are asynchronous.
/// These handlers receive a [`ServerContext`] containing information about
/// the request itself, along with a [`ServerResponseWriter`] that must be used
/// to send the response.
///
/// The default implementation for all async handlers is to respond with
/// [`ErrorCode::ENOSYS`].
///
/// [`fuse_init`]: #method.fuse_init
/// [`ServerContext`]: struct.ServerContext.html
/// [`ServerResponseWriter`]: struct.ServerResponseWriter.html
/// [`ErrorCode::ENOSYS`]: struct.ErrorCode.html#associatedconstant.ENOSYS
pub trait FuseHandlers {
	fn fuse_init(
		&mut self,
		request: &protocol::FuseInitRequest,
	) -> protocol::FuseInitResponse {
		let _ = request;
		protocol::FuseInitResponse::new()
	}

	#[cfg(any(doc, feature = "unstable_access"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_access")))]
	fn access(
		&self,
		ctx: server::ServerContext,
		request: &protocol::AccessRequest,
		respond: impl for<'a> server::Respond<protocol::AccessResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_bmap"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_bmap")))]
	fn bmap(
		&self,
		ctx: server::ServerContext,
		request: &protocol::BmapRequest,
		respond: impl for<'a> server::Respond<protocol::BmapResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_create"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_create")))]
	fn create(
		&self,
		ctx: server::ServerContext,
		request: &protocol::CreateRequest,
		respond: impl for<'a> server::Respond<protocol::CreateResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fallocate"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fallocate")))]
	fn fallocate(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FallocateRequest,
		respond: impl for<'a> server::Respond<protocol::FallocateResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
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

	fn forget(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ForgetRequest,
	) {
		let _ = (ctx, request);
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

	#[cfg(any(doc, feature = "unstable_fsyncdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fsyncdir")))]
	fn fsyncdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncdirRequest,
		respond: impl for<'a> server::Respond<protocol::FsyncdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn getattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetattrRequest,
		respond: impl for<'a> server::Respond<protocol::GetattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_getlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_getlk")))]
	fn getlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetlkRequest,
		respond: impl for<'a> server::Respond<protocol::GetlkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn getxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetxattrRequest,
		respond: impl for<'a> server::Respond<protocol::GetxattrResponse<'a>>,
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

	fn link(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LinkRequest,
		respond: impl for<'a> server::Respond<protocol::LinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn listxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ListxattrRequest,
		respond: impl for<'a> server::Respond<protocol::ListxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn lookup(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LookupRequest,
		respond: impl for<'a> server::Respond<protocol::LookupResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_lseek"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_lseek")))]
	fn lseek(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LseekRequest,
		respond: impl for<'a> server::Respond<protocol::LseekResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn mkdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MkdirRequest,
		respond: impl for<'a> server::Respond<protocol::MkdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn mknod(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MknodRequest,
		respond: impl for<'a> server::Respond<protocol::MknodResponse<'a>>,
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

	fn opendir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpendirRequest,
		respond: impl for<'a> server::Respond<protocol::OpendirResponse<'a>>,
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

	fn readdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReaddirRequest,
		respond: impl for<'a> server::Respond<protocol::ReaddirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn readlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadlinkRequest,
		respond: impl for<'a> server::Respond<protocol::ReadlinkResponse<'a>>,
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

	fn releasedir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleasedirRequest,
		respond: impl for<'a> server::Respond<protocol::ReleasedirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_removexattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_removexattr")))]
	fn removexattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RemovexattrRequest,
		respond: impl for<'a> server::Respond<protocol::RemovexattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn rename(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RenameRequest,
		respond: impl for<'a> server::Respond<protocol::RenameResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn rmdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RmdirRequest,
		respond: impl for<'a> server::Respond<protocol::RmdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_setattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setattr")))]
	fn setattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetattrRequest,
		respond: impl for<'a> server::Respond<protocol::SetattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_setlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setlk")))]
	fn setlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetlkRequest,
		respond: impl for<'a> server::Respond<protocol::SetlkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_setxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setxattr")))]
	fn setxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetxattrRequest,
		respond: impl for<'a> server::Respond<protocol::SetxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_statfs"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_statfs")))]
	fn statfs(
		&self,
		ctx: server::ServerContext,
		request: &protocol::StatfsRequest,
		respond: impl for<'a> server::Respond<protocol::StatfsResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn symlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SymlinkRequest,
		respond: impl for<'a> server::Respond<protocol::SymlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(ErrorCode::ENOSYS);
	}

	fn unlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::UnlinkRequest,
		respond: impl for<'a> server::Respond<protocol::UnlinkResponse<'a>>,
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
