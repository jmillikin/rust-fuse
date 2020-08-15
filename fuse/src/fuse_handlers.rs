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

use crate::internal::errors;
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
/// error code `ENOSYS`.
///
/// [`fuse_init`]: #method.fuse_init
/// [`ServerContext`]: struct.ServerContext.html
/// [`ServerResponseWriter`]: struct.ServerResponseWriter.html
pub trait FuseHandlers {
	/// Initialize the FUSE connection parameters.
	///
	/// Most servers do not need to override this method.
	///
	/// The default implementation returns a response created by the helper
	/// function [`FuseInitResponse::for_request`], which is also a good starting
	/// point for custom implementations.
	///
	/// [`FuseInitResponse::for_request`]: protocol/struct.FuseInitResponse.html#method.for_request
	fn fuse_init(
		&mut self,
		request: &protocol::FuseInitRequest,
	) -> protocol::FuseInitResponse {
		protocol::FuseInitResponse::for_request_impl(request)
	}

	#[cfg(any(doc, feature = "unstable_fuse_access"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_access")))]
	fn access(
		&self,
		ctx: server::ServerContext,
		request: &protocol::AccessRequest,
		respond: impl for<'a> server::RespondOnce<protocol::AccessResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_bmap"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_bmap")))]
	fn bmap(
		&self,
		ctx: server::ServerContext,
		request: &protocol::BmapRequest,
		respond: impl for<'a> server::RespondOnce<protocol::BmapResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_create"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_create")))]
	fn create(
		&self,
		ctx: server::ServerContext,
		request: &protocol::CreateRequest,
		respond: impl for<'a> server::RespondOnce<protocol::CreateResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_fallocate"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_fallocate")))]
	fn fallocate(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FallocateRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FallocateResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_flush"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_flush")))]
	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FlushResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn forget(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ForgetRequest,
	) {
		let _ = (ctx, request);
	}

	#[cfg(any(doc, feature = "unstable_fuse_fsync"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_fsync")))]
	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FsyncResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_fsyncdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_fsyncdir")))]
	fn fsyncdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncdirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FsyncdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn getattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::GetattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_getlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_getlk")))]
	fn getlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetlkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::GetlkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_getxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_getxattr")))]
	fn getxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetxattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::GetxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_ioctl"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_ioctl")))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		respond: impl for<'a> server::RespondOnce<protocol::IoctlResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_link"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_link")))]
	fn link(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::LinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_listxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_listxattr")))]
	fn listxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ListxattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ListxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn lookup(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LookupRequest,
		respond: impl for<'a> server::RespondOnce<protocol::LookupResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_lseek"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_lseek")))]
	fn lseek(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LseekRequest,
		respond: impl for<'a> server::RespondOnce<protocol::LseekResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_mkdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_mkdir")))]
	fn mkdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MkdirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::MkdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_mknod"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_mknod")))]
	fn mknod(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MknodRequest,
		respond: impl for<'a> server::RespondOnce<protocol::MknodResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		respond: impl for<'a> server::RespondOnce<protocol::OpenResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn opendir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpendirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::OpendirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReadResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn readdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReaddirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReaddirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn readlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadlinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReadlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReleaseResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	fn releasedir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleasedirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReleasedirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_removexattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_removexattr")))]
	fn removexattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RemovexattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::RemovexattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_rename"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_rename")))]
	fn rename(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RenameRequest,
		respond: impl for<'a> server::RespondOnce<protocol::RenameResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_rmdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_rmdir")))]
	fn rmdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RmdirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::RmdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_setattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_setattr")))]
	fn setattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SetattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_setlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_setlk")))]
	fn setlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetlkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SetlkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_setxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_setxattr")))]
	fn setxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetxattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SetxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_statfs"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_statfs")))]
	fn statfs(
		&self,
		ctx: server::ServerContext,
		request: &protocol::StatfsRequest,
		respond: impl for<'a> server::RespondOnce<protocol::StatfsResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_symlink"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_symlink")))]
	fn symlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SymlinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SymlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_unlink"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_unlink")))]
	fn unlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::UnlinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::UnlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	#[cfg(any(doc, feature = "unstable_fuse_write"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_write")))]
	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		respond: impl for<'a> server::RespondOnce<protocol::WriteResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}
}
