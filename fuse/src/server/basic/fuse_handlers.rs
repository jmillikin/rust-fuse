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

use crate::io::{ServerSendError as SendError, ServerSocket};
use crate::protocol;
use crate::server::basic::server::{self, SendReply, SentReply};

/// User-provided handlers for FUSE operations.
///
/// Most FUSE handlers (with the exception of [`fuse_init`]) are asynchronous.
/// These handlers receive a [`ServerContext`] containing information about
/// the request itself, along with a [`ServerResponseWriter`] that must be used
/// to send the response.
///
/// The default implementation for all async handlers is to respond with
/// [`Error::UNIMPLEMENTED`].
///
/// [`fuse_init`]: #method.fuse_init
/// [`ServerContext`]: struct.ServerContext.html
/// [`ServerResponseWriter`]: struct.ServerResponseWriter.html
/// [`Error::UNIMPLEMENTED`]: crate::Error::UNIMPLEMENTED
#[allow(unused_variables)]
pub trait FuseHandlers<S: ServerSocket> {
	fn access(
		&self,
		ctx: server::ServerContext,
		request: &protocol::AccessRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::AccessResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_bmap"))]
	fn bmap(
		&self,
		ctx: server::ServerContext,
		request: &protocol::BmapRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::BmapResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn create(
		&self,
		ctx: server::ServerContext,
		request: &protocol::CreateRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::CreateResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn fallocate(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FallocateRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FallocateResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FlushResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn forget(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ForgetRequest,
	) {
		if let Some(hooks) = ctx.hooks {
			hooks.unhandled_request(ctx.header);
		}
	}

	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FsyncResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn fsyncdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncdirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::FsyncdirResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn getattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::GetattrResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn getlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetlkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::GetlkResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn getxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetxattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::GetxattrResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::IoctlResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn link(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::LinkResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn listxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ListxattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ListxattrResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn lookup(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LookupRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::LookupResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn lseek(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LseekRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::LseekResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn mkdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MkdirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::MkdirResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn mknod(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MknodRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::MknodResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::OpenResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn opendir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpendirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::OpendirResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReadResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn readdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReaddirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReaddirResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn readlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadlinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReadlinkResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReleaseResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn releasedir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleasedirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::ReleasedirResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn removexattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RemovexattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::RemovexattrResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn rename(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RenameRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::RenameResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn rmdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RmdirRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::RmdirResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	#[cfg(any(doc, feature = "unstable_setattr"))]
	fn setattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SetattrResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn setlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetlkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SetlkResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn setxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetxattrRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SetxattrResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn statfs(
		&self,
		ctx: server::ServerContext,
		request: &protocol::StatfsRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::StatfsResponse>, SendError<S::Error>> {
		#[cfg(not(target_os = "freebsd"))]
		{
			server::unhandled_request(ctx, send_reply)
		}

		#[cfg(target_os = "freebsd")]
		{
			if let Some(hooks) = ctx.hooks {
				hooks.unhandled_request(ctx.header);
			}
			let resp = protocol::StatfsResponse::new();
			send_reply.ok(&resp)
		}

	}

	fn symlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SymlinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::SymlinkResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn unlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::UnlinkRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::UnlinkResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}

	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		send_reply: impl SendReply<S>,
	) -> Result<SentReply<protocol::WriteResponse>, SendError<S::Error>> {
		server::unhandled_request(ctx, send_reply)
	}
}
