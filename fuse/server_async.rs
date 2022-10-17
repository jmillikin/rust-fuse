// Copyright 2022 John Millikin and the rust-fuse contributors.
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

use core::num;

use crate::operations::cuse_init::{CuseInitRequest, CuseInitResponse};
use crate::operations::fuse_init::{FuseInitRequest, FuseInitResponse};
use crate::server;
use crate::server::encode;

pub use crate::server::{
	CuseRequest,
	CuseRequestOptions,
	CuseResponse,
	CuseResponseOptions,
	FuseRequest,
	FuseRequestOptions,
	FuseResponse,
	FuseResponseOptions,
	Hooks,
	Request,
	RequestError,
	Response,
	ServerError,
};

pub mod io;

pub async fn cuse_init<'a, S: io::CuseSocket>(
	socket: &mut S,
	mut init_fn: impl FnMut(&CuseInitRequest) -> CuseInitResponse<'a>,
) -> Result<CuseInitResponse<'a>, ServerError<S::Error>> {
	let mut buf = crate::io::MinReadBuffer::new();

	loop {
		let recv_len = socket.recv(buf.as_slice_mut()).await?;
		let request = Request::new(buf.as_aligned_slice().truncate(recv_len))?;
		let init_req = CuseInitRequest::from_request(request)?;
		let (response, ok) = server::cuse_handshake(&init_req, &mut init_fn)?;
		let request_id = request.header().request_id();
		let mut header = crate::ResponseHeader::new(request_id);
		socket
			.send(response.to_response(&mut header).into())
			.await?;
		if ok {
			return Ok(response);
		}
	}
}

pub async fn fuse_init<S: io::FuseSocket>(
	socket: &mut S,
	mut init_fn: impl FnMut(&FuseInitRequest) -> FuseInitResponse,
) -> Result<FuseInitResponse, ServerError<S::Error>> {
	let mut buf = crate::io::MinReadBuffer::new();

	loop {
		let recv_len = socket.recv(buf.as_slice_mut()).await?;
		let request = Request::new(buf.as_aligned_slice().truncate(recv_len))?;
		let init_req = FuseInitRequest::from_request(request)?;
		let (response, ok) = server::fuse_handshake(&init_req, &mut init_fn)?;
		let request_id = request.header().request_id();
		let mut header = crate::ResponseHeader::new(request_id);
		socket
			.send(response.to_response(&mut header).into())
			.await?;
		if ok {
			return Ok(response);
		}
	}
}

/// Receive a CUSE or FUSE request from an async [`Socket`].
///
/// Returns `Ok(None)` if the socket's connection has been closed.
///
/// [`Socket`]: io::Socket
pub async fn recv<'a, S: io::Socket>(
	socket: &S,
	mut buf: crate::io::AlignedSliceMut<'a>,
) -> Result<Option<Request<'a>>, ServerError<S::Error>> {
	let recv_len = match socket.recv(buf.get_mut()).await {
		Ok(len) => len,
		Err(io::RecvError::ConnectionClosed(_)) => return Ok(None),
		Err(err) => return Err(err.into()),
	};
	let recv_buf = buf.truncate(recv_len);
	Ok(Some(Request::new(recv_buf.into())?))
}

/// Send an error response to an async [`Socket`].
///
/// [`Socket`]: io::Socket
pub async fn send_error<S: io::Socket>(
	socket: &S,
	request_id: num::NonZeroU64,
	error: crate::Error,
) -> Result<(), io::SendError<S::Error>> {
	let mut response_header = crate::ResponseHeader::new(request_id);
	socket
		.send(encode::error(&mut response_header, error).into())
		.await
}
