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

use core::mem::transmute;

use crate::internal::fuse_kernel;
use crate::io::{Buffer, DecodeError};
use crate::io::decode::{self, DecodeRequest, RequestBuf};
use crate::server::request::RequestHeader;

pub struct FuseRequest<'a> {
	raw: RequestBuf<'a>,
	version_minor: u32,
}

impl<'a> FuseRequest<'a> {
	pub(crate) fn new(
		buf: &'a impl Buffer,
		recv_len: usize,
		version_minor: u32,
	) -> Result<Self, DecodeError> {
		let request_buf = RequestBuf::new(buf, recv_len)?;
		Ok(Self {
			raw: request_buf,
			version_minor,
		})
	}

	pub fn header(&self) -> &'a RequestHeader {
		RequestHeader::from_buf(self.raw)
	}

	pub(crate) fn opcode(&self) -> fuse_kernel::fuse_opcode {
		self.raw.header().opcode
	}

	pub fn operation(&self) -> Option<FuseOperation> {
		use fuse_kernel::fuse_opcode as opcode;
		match self.raw.header().opcode {
			opcode(0)                      => None,
			opcode(7) | opcode(19)         => None,
			fuse_kernel::FUSE_INIT         => None,
			x if x.0 > MAX_FUSE_OPCODE     => None,
			fuse_kernel::FUSE_SETLKW       => Some(FuseOperation::Setlk),
			fuse_kernel::FUSE_BATCH_FORGET => Some(FuseOperation::Forget),
			fuse_kernel::FUSE_RENAME2      => Some(FuseOperation::Rename),
			x => Some(unsafe { transmute(x.0) }),
		}
	}

	pub(crate) fn decode<R>(self) -> Result<R, DecodeError>
	where
		R: DecodeRequest<'a, decode::FUSE>,
	{
		DecodeRequest::decode(self.raw, self.version_minor)
	}
}

const MAX_FUSE_OPCODE: u32 = fuse_kernel::FUSE_LSEEK.0;

#[non_exhaustive]
#[repr(u32)]
pub enum FuseOperation {
	Lookup      = fuse_kernel::FUSE_LOOKUP.0,
	Forget      = fuse_kernel::FUSE_FORGET.0,
	Getattr     = fuse_kernel::FUSE_GETATTR.0,
	Setattr     = fuse_kernel::FUSE_SETATTR.0,
	Readlink    = fuse_kernel::FUSE_READLINK.0,
	Symlink     = fuse_kernel::FUSE_SYMLINK.0,
	Mknod       = fuse_kernel::FUSE_MKNOD.0,
	Mkdir       = fuse_kernel::FUSE_MKDIR.0,
	Unlink      = fuse_kernel::FUSE_UNLINK.0,
	Rmdir       = fuse_kernel::FUSE_RMDIR.0,
	Rename      = fuse_kernel::FUSE_RENAME.0,
	Link        = fuse_kernel::FUSE_LINK.0,
	Open        = fuse_kernel::FUSE_OPEN.0,
	Read        = fuse_kernel::FUSE_READ.0,
	Write       = fuse_kernel::FUSE_WRITE.0,
	Statfs      = fuse_kernel::FUSE_STATFS.0,
	Release     = fuse_kernel::FUSE_RELEASE.0,
	Fsync       = fuse_kernel::FUSE_FSYNC.0,
	Setxattr    = fuse_kernel::FUSE_SETXATTR.0,
	Getxattr    = fuse_kernel::FUSE_GETXATTR.0,
	Listxattr   = fuse_kernel::FUSE_LISTXATTR.0,
	Removexattr = fuse_kernel::FUSE_REMOVEXATTR.0,
	Flush       = fuse_kernel::FUSE_FLUSH.0,
	Opendir     = fuse_kernel::FUSE_OPENDIR.0,
	Readdir     = fuse_kernel::FUSE_READDIR.0,
	Releasedir  = fuse_kernel::FUSE_RELEASEDIR.0,
	Fsyncdir    = fuse_kernel::FUSE_FSYNCDIR.0,
	Getlk       = fuse_kernel::FUSE_GETLK.0,
	Setlk       = fuse_kernel::FUSE_SETLK.0,
	Access      = fuse_kernel::FUSE_ACCESS.0,
	Create      = fuse_kernel::FUSE_CREATE.0,
	Interrupt   = fuse_kernel::FUSE_INTERRUPT.0,
	Bmap        = fuse_kernel::FUSE_BMAP.0,
	Destroy     = fuse_kernel::FUSE_DESTROY.0,
	Ioctl       = fuse_kernel::FUSE_IOCTL.0,
	Poll        = fuse_kernel::FUSE_POLL.0,
	NotifyReply = fuse_kernel::FUSE_NOTIFY_REPLY.0,
	Fallocate   = fuse_kernel::FUSE_FALLOCATE.0,
	Readdirplus = fuse_kernel::FUSE_READDIRPLUS.0,
	Lseek       = fuse_kernel::FUSE_LSEEK.0,
}
