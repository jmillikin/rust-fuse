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

use crate::io::OutputStream;
use crate::protocol;
use crate::server::basic::server::{self, SendReply};

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
pub trait FuseHandlers<S: OutputStream> {
	fn access(
		&self,
		ctx: server::ServerContext,
		request: &protocol::AccessRequest,
		send_reply: impl for<'a> SendReply<S, protocol::AccessResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_bmap"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_bmap")))]
	fn bmap(
		&self,
		ctx: server::ServerContext,
		request: &protocol::BmapRequest,
		send_reply: impl for<'a> SendReply<S, protocol::BmapResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn create(
		&self,
		ctx: server::ServerContext,
		request: &protocol::CreateRequest,
		send_reply: impl for<'a> SendReply<S, protocol::CreateResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn fallocate(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FallocateRequest,
		send_reply: impl for<'a> SendReply<S, protocol::FallocateResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		send_reply: impl for<'a> SendReply<S, protocol::FlushResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn forget(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ForgetRequest,
	) {
	}

	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		send_reply: impl for<'a> SendReply<S, protocol::FsyncResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn fsyncdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncdirRequest,
		send_reply: impl for<'a> SendReply<S, protocol::FsyncdirResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn getattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetattrRequest,
		send_reply: impl for<'a> SendReply<S, protocol::GetattrResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn getlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetlkRequest,
		send_reply: impl for<'a> SendReply<S, protocol::GetlkResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn getxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetxattrRequest,
		send_reply: impl for<'a> SendReply<S, protocol::GetxattrResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_ioctl")))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		send_reply: impl for<'a> SendReply<S, protocol::IoctlResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn link(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LinkRequest,
		send_reply: impl for<'a> SendReply<S, protocol::LinkResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn listxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ListxattrRequest,
		send_reply: impl for<'a> SendReply<S, protocol::ListxattrResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn lookup(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LookupRequest,
		send_reply: impl for<'a> SendReply<S, protocol::LookupResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn lseek(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LseekRequest,
		send_reply: impl for<'a> SendReply<S, protocol::LseekResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn mkdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MkdirRequest,
		send_reply: impl for<'a> SendReply<S, protocol::MkdirResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn mknod(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MknodRequest,
		send_reply: impl for<'a> SendReply<S, protocol::MknodResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		send_reply: impl for<'a> SendReply<S, protocol::OpenResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn opendir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpendirRequest,
		send_reply: impl for<'a> SendReply<S, protocol::OpendirResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		send_reply: impl for<'a> SendReply<S, protocol::ReadResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn readdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReaddirRequest,
		send_reply: impl for<'a> SendReply<S, protocol::ReaddirResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn readlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadlinkRequest,
		send_reply: impl for<'a> SendReply<S, protocol::ReadlinkResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		send_reply: impl for<'a> SendReply<S, protocol::ReleaseResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn releasedir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleasedirRequest,
		send_reply: impl for<'a> SendReply<S, protocol::ReleasedirResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn removexattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RemovexattrRequest,
		send_reply: impl for<'a> SendReply<S, protocol::RemovexattrResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn rename(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RenameRequest,
		send_reply: impl for<'a> SendReply<S, protocol::RenameResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn rmdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RmdirRequest,
		send_reply: impl for<'a> SendReply<S, protocol::RmdirResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_setattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_setattr")))]
	fn setattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetattrRequest,
		send_reply: impl for<'a> SendReply<S, protocol::SetattrResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn setlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetlkRequest,
		send_reply: impl for<'a> SendReply<S, protocol::SetlkResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn setxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetxattrRequest,
		send_reply: impl for<'a> SendReply<S, protocol::SetxattrResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn statfs(
		&self,
		ctx: server::ServerContext,
		request: &protocol::StatfsRequest,
		send_reply: impl for<'a> SendReply<S, protocol::StatfsResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn symlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SymlinkRequest,
		send_reply: impl for<'a> SendReply<S, protocol::SymlinkResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn unlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::UnlinkRequest,
		send_reply: impl for<'a> SendReply<S, protocol::UnlinkResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}

	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		send_reply: impl for<'a> SendReply<S, protocol::WriteResponse<'a>>,
	) {
		server::unhandled_request(ctx, send_reply)
	}
}
