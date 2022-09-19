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

use crate::internal::fuse_kernel;
use crate::io::{Buffer, RequestError};
use crate::io::decode::{RequestDecoder, RequestBuf};
use crate::protocol::UnknownRequest;
use crate::server::request::RequestHeader;

pub struct CuseRequest<'a> {
	pub(crate) buf: RequestBuf<'a>,
	pub(crate) version_minor: u32,
}

impl<'a> CuseRequest<'a> {
	pub(crate) fn new(
		buf: &'a impl Buffer,
		recv_len: usize,
		version_minor: u32,
	) -> Result<Self, RequestError> {
		let request_buf = RequestBuf::new(buf, recv_len)?;
		Ok(Self {
			buf: request_buf,
			version_minor,
		})
	}

	pub(crate) fn decoder(&self) -> RequestDecoder<'a> {
		RequestDecoder::new(self.buf)
	}

	pub fn header(&self) -> &'a RequestHeader {
		RequestHeader::from_buf(self.buf)
	}

	pub fn into_unknown(self) -> UnknownRequest<'a> {
		UnknownRequest::new(self.buf)
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
