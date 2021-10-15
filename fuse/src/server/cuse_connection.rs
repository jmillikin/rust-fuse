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

use core::num::NonZeroU16;

use crate::io;
use crate::io::encode::{ReplyEncoder, SyncSendOnce};
use crate::server::{CuseRequest, Reply, ReplyInfo};

pub struct CuseConnection<Stream> {
	stream: Stream,
	version: io::ProtocolVersion,
}

impl<S> CuseConnection<S> {
	pub fn version(&self) -> io::ProtocolVersion {
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

impl<S: io::InputStream> CuseConnection<S> {
	pub fn recv<'a>(
		&self,
		buf: &'a mut impl io::Buffer,
	) -> Result<Option<CuseRequest<'a>>, io::Error<S::Error>> {
		let recv_len = match self.stream.recv(buf.borrow_mut()) {
			Ok(Some(x)) => x,
			Ok(None) => return Ok(None),
			Err(err) => return Err(io::Error::RecvFail(err)),
		};
		let v_minor = self.version.minor();
		Ok(Some(CuseRequest::new(buf, recv_len, v_minor)?))
	}
}

impl<S: io::OutputStream> CuseConnection<S> {
	pub fn reply_ok(
		&self,
		request_id: u64,
		reply: &impl Reply,
	) -> Result<(), io::Error<S::Error>> {
		let res = reply.send(
			&self.stream,
			ReplyInfo {
				request_id,
				version_minor: self.version.minor(),
			},
		);
		res.map_err(|err| io::Error::SendFail(err))
	}

	pub fn reply_err(
		&self,
		request_id: u64,
		error_code: NonZeroU16,
	) -> Result<(), io::Error<S::Error>> {
		let send = SyncSendOnce::new(&self.stream);
		let enc = ReplyEncoder::new(send, request_id);
		let res = enc.encode_error(error_code);
		res.map_err(|err| io::Error::SendFail(err))
	}
}
