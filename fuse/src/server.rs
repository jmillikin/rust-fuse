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

use core::cmp::{max, min};

#[cfg(feature = "std")]
use std::sync::Arc;

use crate::channel::{self, ChannelError};
use crate::error::{Error, ErrorCode};
use crate::internal::fuse_io;
use crate::internal::fuse_kernel;
use crate::internal::types::ProtocolVersion;
use crate::protocol::common::{RequestHeader, UnknownRequest};

pub trait ServerChannel: channel::Channel {
	fn try_clone(&self) -> Result<Self, Self::Error>
	where
		Self: Sized;
}

pub struct ServerContext {
	header: fuse_kernel::fuse_in_header,
}

impl<'a> ServerContext {
	pub(crate) fn new(header: fuse_kernel::fuse_in_header) -> Self {
		Self { header }
	}

	pub fn request_header(&self) -> &RequestHeader {
		RequestHeader::new_ref(&self.header)
	}
}

#[allow(unused_variables)]
pub trait ServerHooks {
	fn request(&self, request_header: &RequestHeader) {}
	fn unknown_request(&self, request: &UnknownRequest) {}
	fn request_error(&self, request_header: &RequestHeader, err: Error) {}
	fn response_error(&self, code: Option<ErrorCode>) {}
	fn async_channel_error(&self, code: Option<ErrorCode>) {}
}

pub struct NoopServerHooks(());

impl ServerHooks for NoopServerHooks {}

// When calculating the header overhead, the Linux kernel is permissive
// (allowing overheads as small as `size(fuse_in_header + fuse_write_in`)
// but libfuse is conservative (reserving 4 KiB).
//
// This code follows libfuse because I don't understand why such a large
// value was chosen.
const HEADER_OVERHEAD: usize = 4096;

pub(crate) fn read_buf_size(max_write: u32) -> usize {
	let max_write = max_write as usize;

	// The read buffer is the maximum write size, plus a fixed overhead for
	// request headers.
	//
	max(
		HEADER_OVERHEAD + max_write,
		fuse_kernel::FUSE_MIN_READ_BUFFER,
	)
}

#[cfg(not(feature = "std"))]
pub(crate) const fn capped_max_write() -> u32 {
	// In no_std mode the read buffer has a fixed size of FUSE_MIN_READ_BUFFER
	// bytes, so init responses must have their max_write capped to a value such
	// that `read_buf_size(max_write) <= FUSE_MIN_READ_BUFFER`.
	return (fuse_kernel::FUSE_MIN_READ_BUFFER - HEADER_OVERHEAD) as u32;
}

pub(crate) fn negotiate_version(
	kernel: ProtocolVersion,
) -> Option<ProtocolVersion> {
	if kernel.major() != fuse_kernel::FUSE_KERNEL_VERSION {
		return None;
	}
	Some(ProtocolVersion::new(
		fuse_kernel::FUSE_KERNEL_VERSION,
		min(kernel.minor(), fuse_kernel::FUSE_KERNEL_MINOR_VERSION),
	))
}

pub(crate) fn main_loop<Buf, C, Cb>(
	channel: &C,
	read_buf: &mut Buf,
	fuse_version: ProtocolVersion,
	semantics: fuse_io::Semantics,
	cb: Cb,
) -> Result<(), C::Error>
where
	Buf: fuse_io::AlignedBuffer,
	C: channel::Channel,
	Cb: Fn(fuse_io::RequestDecoder) -> Result<(), C::Error>,
{
	loop {
		let request_size = match channel.receive(read_buf.get_mut()) {
			Err(err) => {
				if semantics == fuse_io::Semantics::FUSE {
					if err.error_code() == Some(ErrorCode::ENODEV) {
						return Ok(());
					}
				}
				return Err(err);
			},
			Ok(request_size) => request_size,
		};
		let request_buf = fuse_io::aligned_slice(read_buf, request_size);
		cb(fuse_io::RequestDecoder::new(
			request_buf,
			fuse_version,
			semantics,
		)?)?;
	}
}

pub(crate) trait MaybeSendChannel {
	#[cfg(feature = "std")]
	type T: channel::Channel + Send + Sync + 'static;

	#[cfg(not(feature = "std"))]
	type T: channel::Channel;
}

#[cfg(feature = "std")]
impl<C> MaybeSendChannel for C
where
	C: channel::Channel + Send + Sync + 'static,
{
	type T = C;
}

#[cfg(not(feature = "std"))]
impl<C> MaybeSendChannel for C
where
	C: channel::Channel,
{
	type T = C;
}

pub(crate) trait MaybeSendHooks {
	#[cfg(feature = "std")]
	type T: ServerHooks + Send + Sync + 'static;

	#[cfg(not(feature = "std"))]
	type T: ServerHooks;
}

#[cfg(feature = "std")]
impl<H> MaybeSendHooks for H
where
	H: ServerHooks + Send + Sync + 'static,
{
	type T = H;
}

#[cfg(not(feature = "std"))]
impl<H> MaybeSendHooks for H
where
	H: ServerHooks,
{
	type T = H;
}

mod private {
	pub trait Sealed {}
}

/// **\[SEALED\]**
pub trait Respond<R>: private::Sealed {
	fn ok(self, response: &R);
	fn err(self, err: ErrorCode);

	#[cfg(feature = "std")]
	#[cfg_attr(doc, doc(cfg(feature = "std")))]
	fn into_async(self) -> RespondAsync<R>;
}

pub(crate) struct RespondRef<'a, C, Hooks>
where
	C: channel::Channel,
{
	channel: &'a C,
	hooks: Option<&'a Hooks>,
	channel_err: &'a mut Result<(), C::Error>,
	request_id: u64,
	fuse_version: ProtocolVersion,

	#[cfg(feature = "std")]
	channel_arc: &'a Arc<C>,

	#[cfg(feature = "std")]
	hooks_arc: Option<&'a Arc<Hooks>>,
}

impl<'a, C, Hooks> RespondRef<'a, C, Hooks>
where
	C: channel::Channel,
	Hooks: ServerHooks,
{
	pub(crate) fn new(
		channel: &'a C,
		hooks: Option<&'a Hooks>,
		channel_err: &'a mut Result<(), C::Error>,
		request_id: u64,
		fuse_version: ProtocolVersion,
		#[cfg(feature = "std")] channel_arc: &'a Arc<C>,
		#[cfg(feature = "std")] hooks_arc: Option<&'a Arc<Hooks>>,
	) -> Self {
		Self {
			channel,
			hooks,
			channel_err,
			request_id,
			fuse_version,
			#[cfg(feature = "std")]
			channel_arc,
			#[cfg(feature = "std")]
			hooks_arc,
		}
	}

	pub(crate) fn encoder(&self) -> fuse_io::ResponseEncoder<C> {
		fuse_io::ResponseEncoder::new(
			self.channel,
			self.request_id,
			self.fuse_version,
		)
	}

	fn ok_impl<R>(self, response: &R)
	where
		R: fuse_io::EncodeResponse,
	{
		if let Err(err) = response.encode_response(self.encoder()) {
			if let Some(hooks) = &self.hooks {
				hooks.response_error(err.error_code())
			}
			self.err_impl(ErrorCode::EIO);
		}
	}

	pub(crate) fn err_impl(self, err: ErrorCode) {
		*self.channel_err = self.encoder().encode_error(err);
	}
}

impl<C, Hooks> private::Sealed for RespondRef<'_, C, Hooks> where
	C: channel::Channel
{
}

#[cfg(feature = "std")]
impl<C, Hooks, R> Respond<R> for RespondRef<'_, C, Hooks>
where
	C: channel::Channel + Send + Sync + 'static,
	Hooks: ServerHooks + Send + Sync + 'static,
	R: fuse_io::EncodeResponse,
{
	fn ok(self, response: &R) {
		self.ok_impl(response)
	}

	fn err(self, err: ErrorCode) {
		self.err_impl(err)
	}

	fn into_async(self) -> RespondAsync<R> {
		self.new_respond_async()
	}
}

#[cfg(not(feature = "std"))]
impl<C, Hooks, R> Respond<R> for RespondRef<'_, C, Hooks>
where
	C: channel::Channel,
	Hooks: ServerHooks,
	R: fuse_io::EncodeResponse,
{
	fn ok(self, response: &R) {
		self.ok_impl(response)
	}

	fn err(self, err: ErrorCode) {
		self.err_impl(err)
	}
}

#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub struct RespondAsync<R>(Box<dyn RespondAsyncInner<R> + 'static>);

#[cfg(feature = "std")]
impl<R> RespondAsync<R> {
	pub fn ok(self, response: &R) {
		self.0.ok(response)
	}
	pub fn err(self, err: ErrorCode) {
		self.0.err(err)
	}
}

#[cfg(feature = "std")]
trait RespondAsyncInner<R>: Send + Sync {
	fn ok(&self, response: &R);
	fn err(&self, err: ErrorCode);
}

#[cfg(feature = "std")]
struct RespondAsyncInnerImpl<C, Hooks> {
	channel: Arc<C>,
	hooks: Option<Arc<Hooks>>,
	request_id: u64,
	fuse_version: ProtocolVersion,
}

#[cfg(feature = "std")]
impl<C, Hooks> RespondAsyncInnerImpl<C, Hooks>
where
	C: channel::Channel,
	Hooks: ServerHooks,
{
	fn encoder(&self) -> fuse_io::ResponseEncoder<C> {
		fuse_io::ResponseEncoder::new(
			self.channel.as_ref(),
			self.request_id,
			self.fuse_version,
		)
	}

	fn err_impl(&self, err: ErrorCode) {
		if let Err(err) = self.encoder().encode_error(err) {
			if let Some(hooks) = &self.hooks {
				hooks.async_channel_error(err.error_code())
			}
		}
	}
}

#[cfg(feature = "std")]
impl<C, Hooks, R> RespondAsyncInner<R> for RespondAsyncInnerImpl<C, Hooks>
where
	C: channel::Channel + Send + Sync,
	Hooks: ServerHooks + Send + Sync,
	R: fuse_io::EncodeResponse,
{
	fn ok(&self, response: &R) {
		if let Err(err) = response.encode_response(self.encoder()) {
			if let Some(hooks) = &self.hooks {
				hooks.response_error(err.error_code())
			}
			self.err_impl(ErrorCode::EIO)
		}
	}

	fn err(&self, err: ErrorCode) {
		self.err_impl(err)
	}
}

#[cfg(feature = "std")]
impl<'a, C, Hooks> RespondRef<'a, C, Hooks>
where
	C: channel::Channel + Send + Sync + 'static,
	Hooks: ServerHooks + Send + Sync + 'static,
{
	fn new_respond_async<R>(self) -> RespondAsync<R>
	where
		R: fuse_io::EncodeResponse,
	{
		RespondAsync(Box::new(RespondAsyncInnerImpl {
			channel: self.channel_arc.clone(),
			hooks: self.hooks_arc.map(|h| h.clone()),
			request_id: self.request_id,
			fuse_version: self.fuse_version,
		}))
	}
}
