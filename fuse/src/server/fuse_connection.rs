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
use core::num::NonZeroU16;

use crate::io::{
	AlignedSlice,
	Buffer,
	InputStream,
	OutputStream,
	ProtocolVersion,
};
use crate::io::encode::{ReplyEncoder, SyncSendOnce};
use crate::server::{FuseRequest, Recv, Reply, ReplyInfo};

pub struct FuseConnection<Stream> {
	stream: Stream,
	version: ProtocolVersion,
}

impl<S> FuseConnection<S> {
	pub fn version(&self) -> ProtocolVersion {
		self.version
	}

	pub fn try_clone<Error>(
		&self,
		clone_stream: impl FnOnce(&S) -> Result<S, Error>,
	) -> Result<Self, Error> {
		Ok(Self {
			stream: clone_stream(&self.stream)?,
			version: self.version,
		})
	}
}

impl<S: InputStream> FuseConnection<S> {
	pub fn recv<'a>(
		&self,
		buf: &'a mut impl Buffer,
	) -> Result<Option<Recv<'a, FuseRequest<'a>>>, S::Error> {
		use crate::server::request::RecvBuf;

		let recv_len = match self.stream.recv(buf.borrow_mut())? {
			None => return Ok(None),
			Some(x) => x,
		};
		Ok(Some(Recv {
			buf: RecvBuf::Raw(AlignedSlice::new(buf), recv_len),
			version_minor: self.version.minor(),
			_phantom: PhantomData,
		}))
	}
}

impl<S: OutputStream> FuseConnection<S> {
	pub fn reply_ok(
		&self,
		request_id: u64,
		reply: &impl Reply,
	) -> Result<(), S::Error> {
		reply.send(
			&self.stream,
			ReplyInfo {
				request_id,
				version_minor: self.version.minor(),
			},
		)
	}

	pub fn reply_err(
		&self,
		request_id: u64,
		error_code: NonZeroU16,
	) -> Result<(), S::Error> {
		let send = SyncSendOnce::new(&self.stream);
		let enc = ReplyEncoder::new(send, request_id);
		enc.encode_error(error_code)
	}
}
