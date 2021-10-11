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
use core::num::NonZeroUsize;

use crate::internal::fuse_kernel;
use crate::io::{Buffer, DecodeError};
use crate::io::decode::RequestBuf;
use crate::server::request::{Request, RequestHeader};

pub struct CuseRequest<'a> {
	buf: RequestBuf<'a>,
	version_minor: u32,
}

impl<'a> CuseRequest<'a> {
	pub(crate) fn new(
		buf: &'a impl Buffer,
		recv_len: NonZeroUsize,
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

	pub fn operation(&self) -> Option<CuseOperation> {
		use CuseOperation as Op;
		match self.buf.header().opcode {
			fuse_kernel::FUSE_OPEN    => Some(Op::Open),
			fuse_kernel::FUSE_READ    => Some(Op::Read),
			fuse_kernel::FUSE_WRITE   => Some(Op::Write),
			fuse_kernel::FUSE_RELEASE => Some(Op::Release),
			fuse_kernel::FUSE_FSYNC   => Some(Op::Fsync),
			fuse_kernel::FUSE_FLUSH   => Some(Op::Flush),
			fuse_kernel::FUSE_DESTROY => Some(Op::Destroy),
			fuse_kernel::FUSE_IOCTL   => Some(Op::Ioctl),
			fuse_kernel::FUSE_POLL    => Some(Op::Poll),
			_ => None,
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

#[non_exhaustive]
#[repr(u32)]
pub enum CuseOperation {
	Open    = fuse_kernel::FUSE_OPEN.0,
	Read    = fuse_kernel::FUSE_READ.0,
	Write   = fuse_kernel::FUSE_WRITE.0,
	Release = fuse_kernel::FUSE_RELEASE.0,
	Fsync   = fuse_kernel::FUSE_FSYNC.0,
	Flush   = fuse_kernel::FUSE_FLUSH.0,
	Destroy = fuse_kernel::FUSE_DESTROY.0,
	Ioctl   = fuse_kernel::FUSE_IOCTL.0,
	Poll    = fuse_kernel::FUSE_POLL.0,
}

mod impls {
	use crate::io::DecodeError;
	use crate::io::decode::{self, DecodeRequest, RequestBuf};
	use crate::protocol::*;
	use crate::server::request::{Recv, RecvBuf, Request};

	use super::CuseRequest;

	fn decode_impl<'a, T: DecodeRequest<'a, decode::CUSE>>(
		recv: Recv<'a, CuseRequest>,
	) -> Result<T, DecodeError> {
		let buf = match recv.buf {
			RecvBuf::Raw(slice, len) => RequestBuf::from_slice(slice, len)?,
			RecvBuf::Decoded(buf) => buf,
		};
		DecodeRequest::<decode::CUSE>::decode(buf, recv.version_minor)
	}

	type RecvCuse<'a> = Recv<'a, CuseRequest<'a>>;

	macro_rules! cuse_request {
		($t:ty) => {
			impl<'a> Request<'a, CuseRequest<'a>> for $t {
				fn decode(recv: RecvCuse<'a>) -> Result<Self, DecodeError> {
					decode_impl(recv)
				}
			}
		};
	}

	impl<'a> Request<'a, CuseRequest<'a>> for CuseRequest<'a> {
		fn decode(
			recv: Recv<'a, CuseRequest<'a>>,
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

	cuse_request! { OpenRequest<'a>    }
	cuse_request! { ReadRequest<'a>    }
	cuse_request! { WriteRequest<'a>   }
	cuse_request! { ReleaseRequest<'a> }
	cuse_request! { FsyncRequest<'a>   }
	cuse_request! { FlushRequest<'a>   }
	cuse_request! { UnknownRequest<'a> }

	#[cfg(any(doc, feature = "unstable_ioctl"))]
	cuse_request! { IoctlRequest<'a> }
}
