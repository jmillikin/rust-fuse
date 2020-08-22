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
#[allow(unused_variables)]
pub trait FuseHandlers {
	fn fuse_init(
		&mut self,
		request: &protocol::FuseInitRequest,
	) -> protocol::FuseInitResponse {
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
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_bmap"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_bmap")))]
	fn bmap(
		&self,
		ctx: server::ServerContext,
		request: &protocol::BmapRequest,
		respond: impl for<'a> server::Respond<protocol::BmapResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_create"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_create")))]
	fn create(
		&self,
		ctx: server::ServerContext,
		request: &protocol::CreateRequest,
		respond: impl for<'a> server::Respond<protocol::CreateResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_fallocate"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fallocate")))]
	fn fallocate(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FallocateRequest,
		respond: impl for<'a> server::Respond<protocol::FallocateResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_flush"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_flush")))]
	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		respond: impl for<'a> server::Respond<protocol::FlushResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn forget(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ForgetRequest,
	) {
	}

	#[cfg(any(doc, feature = "unstable_fsync"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fsync")))]
	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		respond: impl for<'a> server::Respond<protocol::FsyncResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_fsyncdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fsyncdir")))]
	fn fsyncdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncdirRequest,
		respond: impl for<'a> server::Respond<protocol::FsyncdirResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn getattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetattrRequest,
		respond: impl for<'a> server::Respond<protocol::GetattrResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_getlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_getlk")))]
	fn getlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetlkRequest,
		respond: impl for<'a> server::Respond<protocol::GetlkResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn getxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetxattrRequest,
		respond: impl for<'a> server::Respond<protocol::GetxattrResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_ioctl")))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		respond: impl for<'a> server::Respond<protocol::IoctlResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn link(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LinkRequest,
		respond: impl for<'a> server::Respond<protocol::LinkResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn listxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ListxattrRequest,
		respond: impl for<'a> server::Respond<protocol::ListxattrResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn lookup(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LookupRequest,
		respond: impl for<'a> server::Respond<protocol::LookupResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_lseek"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_lseek")))]
	fn lseek(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LseekRequest,
		respond: impl for<'a> server::Respond<protocol::LseekResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn mkdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MkdirRequest,
		respond: impl for<'a> server::Respond<protocol::MkdirResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn mknod(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MknodRequest,
		respond: impl for<'a> server::Respond<protocol::MknodResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		respond: impl for<'a> server::Respond<protocol::OpenResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn opendir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpendirRequest,
		respond: impl for<'a> server::Respond<protocol::OpendirResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		respond: impl for<'a> server::Respond<protocol::ReadResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn readdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReaddirRequest,
		respond: impl for<'a> server::Respond<protocol::ReaddirResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn readlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadlinkRequest,
		respond: impl for<'a> server::Respond<protocol::ReadlinkResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		respond: impl for<'a> server::Respond<protocol::ReleaseResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn releasedir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleasedirRequest,
		respond: impl for<'a> server::Respond<protocol::ReleasedirResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_removexattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_removexattr")))]
	fn removexattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RemovexattrRequest,
		respond: impl for<'a> server::Respond<protocol::RemovexattrResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn rename(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RenameRequest,
		respond: impl for<'a> server::Respond<protocol::RenameResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn rmdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RmdirRequest,
		respond: impl for<'a> server::Respond<protocol::RmdirResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_setattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setattr")))]
	fn setattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetattrRequest,
		respond: impl for<'a> server::Respond<protocol::SetattrResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_setlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setlk")))]
	fn setlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetlkRequest,
		respond: impl for<'a> server::Respond<protocol::SetlkResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_setxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setxattr")))]
	fn setxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetxattrRequest,
		respond: impl for<'a> server::Respond<protocol::SetxattrResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	#[cfg(any(doc, feature = "unstable_statfs"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_statfs")))]
	fn statfs(
		&self,
		ctx: server::ServerContext,
		request: &protocol::StatfsRequest,
		respond: impl for<'a> server::Respond<protocol::StatfsResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn symlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SymlinkRequest,
		respond: impl for<'a> server::Respond<protocol::SymlinkResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn unlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::UnlinkRequest,
		respond: impl for<'a> server::Respond<protocol::UnlinkResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}

	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		respond: impl for<'a> server::Respond<protocol::WriteResponse<'a>>,
	) {
		server::unhandled_request(respond);
	}
}
