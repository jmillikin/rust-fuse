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

use core::marker::PhantomData;
use core::mem::transmute;

use crate::internal::fuse_kernel;
use crate::io::{Buffer, DecodeError};
use crate::io::decode::RequestBuf;
use crate::server::request::{Request, RequestHeader};

pub struct FuseRequest<'a> {
	buf: RequestBuf<'a>,
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
			buf: request_buf,
			version_minor,
		})
	}

	pub fn header(&self) -> &'a RequestHeader {
		RequestHeader::from_buf(self.buf)
	}

	pub fn operation(&self) -> Option<FuseOperation> {
		use fuse_kernel::fuse_opcode as opcode;
		match self.buf.header().opcode {
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

	pub fn decode<R>(self) -> Result<R, DecodeError>
	where
		R: Request<'a, Self>,
	{
		use crate::server::request::{Recv, RecvBuf};

		Request::decode(Recv {
			buf: RecvBuf::Decoded(self.buf),
			version_minor: self.version_minor,
			_phantom: PhantomData,
		})
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

mod impls {
	use crate::io::DecodeError;
	use crate::io::decode::{self, DecodeRequest, RequestBuf};
	use crate::protocol::*;
	use crate::server::request::{Recv, RecvBuf, Request};

	use super::FuseRequest;

	fn decode_impl<'a, T: DecodeRequest<'a, decode::FUSE>>(
		recv: Recv<'a, FuseRequest>,
	) -> Result<T, DecodeError> {
		let buf = match recv.buf {
			RecvBuf::Raw(slice, len) => RequestBuf::from_slice(slice, len)?,
			RecvBuf::Decoded(buf) => buf,
		};
		DecodeRequest::<decode::FUSE>::decode(buf, recv.version_minor)
	}

	type RecvFuse<'a> = Recv<'a, FuseRequest<'a>>;

	macro_rules! fuse_request {
		($t:ty) => {
			impl<'a> Request<'a, FuseRequest<'a>> for $t {
				fn decode(recv: RecvFuse<'a>) -> Result<Self, DecodeError> {
					decode_impl(recv)
				}
			}
		};
	}

	impl<'a> Request<'a, FuseRequest<'a>> for FuseRequest<'a> {
		fn decode(
			recv: Recv<'a, FuseRequest<'a>>,
		) -> Result<Self, DecodeError> {
			let buf = match recv.buf {
				RecvBuf::Raw(slice, len) => RequestBuf::from_slice(slice, len)?,
				RecvBuf::Decoded(buf) => buf,
			};
			Ok(Self {
				buf,
				version_minor: recv.version_minor,
			})
		}
	}

	fuse_request! { AccessRequest<'a>      }
	fuse_request! { CreateRequest<'a>      }
	fuse_request! { FallocateRequest<'a>   }
	fuse_request! { FlushRequest<'a>       }
	fuse_request! { ForgetRequest<'a>      }
	fuse_request! { FsyncRequest<'a>       }
	fuse_request! { FsyncdirRequest<'a>    }
	fuse_request! { GetattrRequest<'a>     }
	fuse_request! { GetxattrRequest<'a>    }
	fuse_request! { GetlkRequest<'a>       }
	fuse_request! { LinkRequest<'a>        }
	fuse_request! { ListxattrRequest<'a>   }
	fuse_request! { LookupRequest<'a>      }
	fuse_request! { LseekRequest<'a>       }
	fuse_request! { MkdirRequest<'a>       }
	fuse_request! { MknodRequest<'a>       }
	fuse_request! { OpenRequest<'a>        }
	fuse_request! { OpendirRequest<'a>     }
	fuse_request! { ReadRequest<'a>        }
	fuse_request! { ReaddirRequest<'a>     }
	fuse_request! { ReadlinkRequest<'a>    }
	fuse_request! { ReleaseRequest<'a>     }
	fuse_request! { ReleasedirRequest<'a>  }
	fuse_request! { RemovexattrRequest<'a> }
	fuse_request! { RenameRequest<'a>      }
	fuse_request! { RmdirRequest<'a>       }
	fuse_request! { SetlkRequest<'a>       }
	fuse_request! { SetxattrRequest<'a>    }
	fuse_request! { StatfsRequest<'a>      }
	fuse_request! { SymlinkRequest<'a>     }
	fuse_request! { UnlinkRequest<'a>      }
	fuse_request! { WriteRequest<'a>       }
	fuse_request! { UnknownRequest<'a>     }

	#[cfg(any(doc, feature = "unstable_bmap"))]
	fuse_request! { BmapRequest<'a> }

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	fuse_request! { IoctlRequest<'a> }

	#[cfg(any(doc, feature = "unstable_setattr"))]
	fuse_request! { SetattrRequest<'a> }
}
