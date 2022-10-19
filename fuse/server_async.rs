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

use crate::cuse;
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

// CuseConnection {{{

/// Represents an active connection to a CUSE client.
pub struct CuseConnection<S> {
	socket: S,
	request_options: CuseRequestOptions,
	response_options: CuseResponseOptions,
}

impl<S: io::CuseSocket> CuseConnection<S> {
	/// Perform a CUSE connection handshake.
	///
	/// When a CUSE connection is being established the client will send a
	/// [`CuseInitRequest`] and the server responds with a [`CuseInitResponse`].
	///
	/// The response specifies the name and device number of the CUSE device that
	/// will be created for this server.
	pub async fn connect<F>(
		socket: S,
		device_name: &cuse::DeviceName,
		device_number: cuse::DeviceNumber,
		mut init_fn: F,
	) -> Result<CuseConnection<S>, ServerError<S::Error>>
	where
		F: FnMut(&CuseInitRequest, &mut CuseInitResponse),
	{
		let mut buf = crate::io::MinReadBuffer::new();

		loop {
			let recv_len = socket.recv(buf.as_slice_mut()).await?;
			let recv_buf = buf.as_aligned_slice().truncate(recv_len);
			let request = Request::new(recv_buf)?;
			let init_req = CuseInitRequest::from_request(request)?;

			let (response, ok) = server::cuse_handshake(&init_req, || {
				let mut response = CuseInitResponse::new(device_name);
				response.set_device_number(device_number);
				init_fn(&init_req, &mut response);
				response
			})?;

			let request_id = request.header().request_id();
			let mut header = crate::ResponseHeader::new(request_id);
			socket
				.send(response.to_response(&mut header).into())
				.await?;

			if !ok {
				continue;
			}

			let req_opts = CuseRequestOptions::from_init_response(&response);
			let resp_opts = CuseResponseOptions::from_init_response(&response);
			return Ok(Self {
				socket,
				request_options: req_opts,
				response_options: resp_opts,
			});
		}
	}

	/// Receive a CUSE request from the client.
	pub async fn recv<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Request<'a>, ServerError<S::Error>> {
		use crate::io::AlignedSlice;
		let recv_len = self.socket.recv(buf.get_mut()).await?;
		let recv_buf = AlignedSlice::from(buf).truncate(recv_len);
		Ok(Request::new(recv_buf)?)
	}

	/// Send a CUSE response to the client.
	pub async fn send<R: CuseResponse>(
		&self,
		request_id: num::NonZeroU64,
		response: &R,
	) -> Result<(), io::SendError<S::Error>> {
		let mut response_header = crate::ResponseHeader::new(request_id);
		let response_buf = response
			.to_response(&mut response_header, self.response_options)
			.into();
		self.socket.send(response_buf).await
	}

	/// Send an error response to the client.
	pub async fn send_error(
		&self,
		request_id: num::NonZeroU64,
		error: crate::Error,
	) -> Result<(), io::SendError<S::Error>> {
		send_error(&self.socket, request_id, error).await
	}
}

impl<S> CuseConnection<S> {
	/// Returns a reference to the underlying [`Socket`] for this connection.
	///
	/// [`Socket`]: io::Socket
	#[inline]
	#[must_use]
	pub fn socket(&self) -> &S {
		&self.socket
	}

	/// Returns the request options for this connection.
	///
	/// Request options are used for decoding requests according to the
	/// current protocol version and negotiated server features.
	#[inline]
	#[must_use]
	pub fn request_options(&self) -> CuseRequestOptions {
		self.request_options
	}

	/// Returns the response options for this connection.
	///
	/// Response options are used for encoding responses according to the
	/// current protocol version and negotiated server features.
	#[inline]
	#[must_use]
	pub fn response_options(&self) -> CuseResponseOptions {
		self.response_options
	}
}

// }}}

// FuseConnection {{{

/// Represents an active connection to a FUSE client.
pub struct FuseConnection<S> {
	socket: S,
	request_options: FuseRequestOptions,
	response_options: FuseResponseOptions,
}

impl<S: io::FuseSocket> FuseConnection<S> {
	/// Perform a FUSE connection handshake.
	///
	/// When a FUSE session is being established the client will send a
	/// [`FuseInitRequest`] and the server responds with a [`FuseInitResponse`].
	///
	/// The response specifies tunable parameters and optional features of the
	/// filesystem server.
	pub async fn connect<F>(
		socket: S,
		mut init_fn: F,
	) -> Result<FuseConnection<S>, ServerError<S::Error>>
	where
		F: FnMut(&FuseInitRequest, &mut FuseInitResponse),
	{
		let mut buf = crate::io::MinReadBuffer::new();

		loop {
			let recv_len = socket.recv(buf.as_slice_mut()).await?;
			let recv_buf = buf.as_aligned_slice().truncate(recv_len);
			let request = Request::new(recv_buf)?;
			let init_req = FuseInitRequest::from_request(request)?;

			let (response, ok) = server::fuse_handshake(&init_req, || {
				let mut response = FuseInitResponse::new();
				init_fn(&init_req, &mut response);
				response
			})?;

			let request_id = request.header().request_id();
			let mut header = crate::ResponseHeader::new(request_id);
			socket
				.send(response.to_response(&mut header).into())
				.await?;

			if !ok {
				continue;
			}

			let req_opts = FuseRequestOptions::from_init_response(&response);
			let resp_opts = FuseResponseOptions::from_init_response(&response);
			return Ok(Self {
				socket,
				request_options: req_opts,
				response_options: resp_opts,
			});
		}
	}

	/// Receive a FUSE request from the client.
	///
	/// Returns `Ok(None)` if the socket's connection has been closed.
	pub async fn recv<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Option<Request<'a>>, ServerError<S::Error>> {
		use crate::io::AlignedSlice;
		let recv_len = match self.socket.recv(buf.get_mut()).await {
			Ok(len) => len,
			Err(io::RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(err) => return Err(err.into()),
		};
		let recv_buf = AlignedSlice::from(buf).truncate(recv_len);
		Ok(Some(Request::new(recv_buf)?))
	}

	/// Send a FUSE response to the client.
	pub async fn send<R: FuseResponse>(
		&self,
		request_id: num::NonZeroU64,
		response: &R,
	) -> Result<(), io::SendError<S::Error>> {
		let mut response_header = crate::ResponseHeader::new(request_id);
		let response_buf = response
			.to_response(&mut response_header, self.response_options)
			.into();
		self.socket.send(response_buf).await
	}

	/// Send an error response to the client.
	pub async fn send_error(
		&self,
		request_id: num::NonZeroU64,
		error: crate::Error,
	) -> Result<(), io::SendError<S::Error>> {
		send_error(&self.socket, request_id, error).await
	}
}

impl<S> FuseConnection<S> {
	/// Returns a reference to the underlying [`Socket`] for this connection.
	///
	/// [`Socket`]: io::Socket
	#[inline]
	#[must_use]
	pub fn socket(&self) -> &S {
		&self.socket
	}

	/// Returns the request options for this connection.
	///
	/// Request options are used for decoding requests according to the
	/// current protocol version and negotiated server features.
	#[inline]
	#[must_use]
	pub fn request_options(&self) -> FuseRequestOptions {
		self.request_options
	}

	/// Returns the response options for this connection.
	///
	/// Response options are used for encoding responses according to the
	/// current protocol version and negotiated server features.
	#[inline]
	#[must_use]
	pub fn response_options(&self) -> FuseResponseOptions {
		self.response_options
	}
}

// }}}

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
