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

//! Implements the `FUSE_IOCTL` operation.

use core::fmt;
use core::marker::PhantomData;
use core::mem::size_of;

use crate::NodeId;
use crate::internal::fuse_kernel;
use crate::server;
use crate::server::io;
use crate::server::io::decode;
use crate::server::io::encode;

use crate::protocol::common::DebugHexU32;
use crate::protocol::common::DebugHexU64;

// IoctlRequest {{{

/// Request type for `FUSE_IOCTL`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_IOCTL` operation.
pub struct IoctlRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_ioctl_in,
	input: &'a [u8],
}

impl<'a> IoctlRequest<'a> {
	pub fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		Self::decode(request.decoder(), false)
	}

	pub fn from_cuse_request(
		request: &server::CuseRequest<'a>,
	) -> Result<Self, io::RequestError> {
		Self::decode(request.decoder(), true)
	}

	#[must_use]
	pub fn node_id(&self) -> NodeId {
		match NodeId::new(self.header.nodeid) {
			Some(id) => id,
			None => crate::ROOT_ID,
		}
	}

	#[must_use]
	pub const fn handle(&self) -> u64 {
		self.body.fh
	}

	#[must_use]
	pub const fn command(&self) -> IoctlCmd {
		IoctlCmd { cmd: self.body.cmd }
	}

	#[must_use]
	pub const fn arg(&self) -> IoctlArg {
		IoctlArg { arg: self.body.arg }
	}

	#[must_use]
	pub const fn input(&self) -> IoctlInput<'a> {
		IoctlInput { bytes: self.input }
	}

	#[must_use]
	pub const fn input_len(&self) -> u32 {
		self.body.in_size
	}

	#[must_use]
	pub const fn output_len(&self) -> u32 {
		self.body.out_size
	}

	#[must_use]
	pub fn flags(&self) -> IoctlRequestFlags {
		IoctlRequestFlags {
			bits: self.body.flags,
		}
	}
}

impl fmt::Debug for IoctlRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("IoctlRequest")
			.field("node_id", &self.node_id())
			.field("handle", &self.body.fh)
			.field("command", &DebugHexU32(self.body.cmd))
			.field("arg", &DebugHexU64(self.body.arg))
			.field("output_len", &self.body.out_size)
			.field("flags", &self.flags())
			.field("input", &self.input)
			.finish()
	}
}

impl<'a> IoctlRequest<'a> {
	fn decode(
		mut dec: decode::RequestDecoder<'a>,
		is_cuse: bool,
	) -> Result<Self, io::RequestError> {
		dec.expect_opcode(fuse_kernel::FUSE_IOCTL)?;

		let header = dec.header();
		let body: &'a fuse_kernel::fuse_ioctl_in = dec.next_sized()?;

		if !is_cuse {
			decode::node_id(header.nodeid)?;
		};

		let input = dec.next_bytes(body.in_size)?;
		Ok(Self {
			header,
			body,
			input,
		})
	}
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IoctlCmd {
	cmd: u32,
}

impl IoctlCmd {
	#[must_use]
	pub const fn new(cmd: u32) -> Self {
		IoctlCmd { cmd }
	}

	#[must_use]
	pub const fn get(self) -> u32 {
		self.cmd
	}
}

impl fmt::Debug for IoctlCmd {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_tuple("IoctlCmd")
			.field(&DebugHexU32(self.cmd))
			.finish()
	}
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IoctlArg {
	arg: u64,
}

impl IoctlArg {
	#[must_use]
	pub const fn new(arg: u64) -> Self {
		Self { arg }
	}

	#[must_use]
	pub const fn get(self) -> u64 {
		self.arg
	}

	#[must_use]
	pub const fn as_ptr<T>(self) -> IoctlPtr<T> {
		IoctlPtr::new(self.arg)
	}
}

impl fmt::Debug for IoctlArg {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_tuple("IoctlArg")
			.field(&DebugHexU64(self.arg))
			.finish()
	}
}

#[repr(C)]
pub struct IoctlPtr<T> {
	addr: u64,
	target: PhantomData<*const T>,
}

impl<T> IoctlPtr<T> {
	#[must_use]
	pub const fn new(addr: u64) -> IoctlPtr<T> {
		IoctlPtr {
			addr,
			target: PhantomData,
		}
	}

	#[must_use]
	pub const fn addr(&self) -> u64 {
		self.addr
	}
}

impl<T> fmt::Debug for IoctlPtr<T> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("IoctlPtr")
			.field("addr", &DebugHexU64(self.addr))
			.field("len", &size_of::<T>())
			.finish()
	}
}

#[derive(Clone, Copy)]
pub struct IoctlInput<'a> {
	bytes: &'a [u8],
}

impl fmt::Debug for IoctlInput<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_tuple("IoctlInput").field(&self.bytes).finish()
	}
}

impl<'a> IoctlInput<'a> {
	#[must_use]
	pub const fn new(bytes: &'a [u8]) -> Self {
		Self { bytes }
	}

	#[must_use]
	pub const fn as_bytes(&self) -> &'a [u8] {
		self.bytes
	}

	#[must_use]
	pub const fn reader(&self) -> IoctlInputReader<'a> {
		IoctlInputReader { bytes: self.bytes }
	}
}

pub struct IoctlInputReader<'a> {
	bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum IoctlInputError {
	UnexpectedEof,
}

impl<'a> IoctlInputReader<'a> {
	pub fn read(&mut self, len: usize) -> Result<&'a [u8], IoctlInputError> {
		if len == self.bytes.len() {
			let out = self.bytes;
			self.bytes = b"";
			return Ok(out);
		}
		if len > self.bytes.len() {
			return Err(IoctlInputError::UnexpectedEof);
		}
		let (out, remainder) = self.bytes.split_at(len);
		self.bytes = remainder;
		Ok(out)
	}

	pub fn read_array<const N: usize>(
		&mut self,
	) -> Result<&'a [u8; N], IoctlInputError> {
		let bytes = self.read(N)?;
		Ok(unsafe { &*(bytes.as_ptr().cast::<[u8; N]>()) })
	}

	#[allow(clippy::missing_safety_doc)] // TODO
	pub unsafe fn read_transmute<T>(
		&mut self,
	) -> Result<&'a T, IoctlInputError> {
		let bytes = self.read(size_of::<T>())?;
		Ok(&*(bytes.as_ptr().cast::<T>()))
	}
}

// }}}

// IoctlResponse {{{

/// Response type for `FUSE_IOCTL`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_IOCTL` operation.
pub struct IoctlResponse<'a> {
	raw: fuse_kernel::fuse_ioctl_out,
	output: IoctlResponseOutput<'a>,
	set_result: bool,
}

enum IoctlResponseOutput<'a> {
	Bytes(&'a [u8]),
	Retry(&'a [IoctlSlice], &'a [IoctlSlice]),
}

impl<'a> IoctlResponse<'a> {
	#[must_use]
	pub fn new(output: &'a [u8]) -> IoctlResponse<'a> {
		Self {
			raw: fuse_kernel::fuse_ioctl_out::zeroed(),
			output: IoctlResponseOutput::Bytes(output),
			set_result: false,
		}
	}

	#[must_use]
	pub fn new_retry(retry: IoctlRetry<'a>) -> IoctlResponse<'a> {
		Self {
			raw: fuse_kernel::fuse_ioctl_out::zeroed(),
			output: IoctlResponseOutput::Retry(
				retry.input_slices,
				retry.output_slices,
			),
			set_result: false,
		}
	}

	#[must_use]
	pub const fn result(&self) -> i32 {
		self.raw.result
	}

	pub fn set_result(&mut self, result: i32) {
		self.raw.result = result;
		self.set_result = true;
	}
}

response_send_funcs!(IoctlResponse<'_>);

impl fmt::Debug for IoctlResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut dbg = fmt.debug_struct("IoctlResponse");
		match self.output {
			IoctlResponseOutput::Bytes(output) => {
				dbg.field("result", &self.raw.result);
				dbg.field("output", &output);
			},
			IoctlResponseOutput::Retry(input_slices, output_slices) => {
				if self.set_result {
					dbg.field("result", &self.raw.result);
				}
				dbg.field(
					"retry",
					&IoctlRetry {
						input_slices,
						output_slices,
					},
				);
			},
		}
		dbg.finish()
	}
}

impl IoctlResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);

		fn raw_bytes<'a>(raw: &'a fuse_kernel::fuse_ioctl_out) -> &'a [u8] {
			unsafe {
				core::slice::from_raw_parts(
					(raw as *const fuse_kernel::fuse_ioctl_out).cast::<u8>(),
					size_of::<fuse_kernel::fuse_ioctl_out>(),
				)
			}
		}

		match self.output {
			IoctlResponseOutput::Bytes(output) => {
				enc.encode_bytes_2(raw_bytes(&self.raw), output)
			},
			IoctlResponseOutput::Retry(input_slices, output_slices) => {
				let mut raw = self.raw;
				raw.flags = fuse_kernel::FUSE_IOCTL_RETRY;
				raw.in_iovs = input_slices.len() as u32;
				raw.out_iovs = output_slices.len() as u32;

				let bytes_2: &[u8] = unsafe {
					core::slice::from_raw_parts(
						input_slices.as_ptr().cast::<u8>(),
						size_of::<IoctlSlice>() * input_slices.len(),
					)
				};
				let bytes_3: &[u8] = unsafe {
					core::slice::from_raw_parts(
						output_slices.as_ptr().cast::<u8>(),
						size_of::<IoctlSlice>() * output_slices.len(),
					)
				};
				enc.encode_bytes_4(raw_bytes(&raw), bytes_2, bytes_3, b"")
			},
		}
	}
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct IoctlSlice {
	base: u64,
	len: u64,
}

impl IoctlSlice {
	#[must_use]
	pub const fn new(base: u64, len: u64) -> IoctlSlice {
		Self { base, len }
	}
}

impl fmt::Debug for IoctlSlice {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("IoctlSlice")
			.field("base", &DebugHexU64(self.base))
			.field("len", &self.len)
			.finish()
	}
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum IoctlRetryError {
	TooManySlices,
}

#[derive(Debug)]
pub struct IoctlRetry<'a> {
	input_slices: &'a [IoctlSlice],
	output_slices: &'a [IoctlSlice],
}

impl<'a> IoctlRetry<'a> {
	pub const fn new(
		input_slices: &'a [IoctlSlice],
		output_slices: &'a [IoctlSlice],
	) -> Result<IoctlRetry<'a>, IoctlRetryError> {
		let input_len = input_slices.len();
		let output_len = output_slices.len();
		if input_len > fuse_kernel::FUSE_IOCTL_MAX_IOV {
			return Err(IoctlRetryError::TooManySlices);
		}
		if output_len > fuse_kernel::FUSE_IOCTL_MAX_IOV {
			return Err(IoctlRetryError::TooManySlices);
		}
		if input_len + output_len > fuse_kernel::FUSE_IOCTL_MAX_IOV {
			return Err(IoctlRetryError::TooManySlices);
		}

		Ok(Self {
			input_slices,
			output_slices,
		})
	}

	#[must_use]
	pub const fn input_slices(&self) -> &'a [IoctlSlice] {
		self.input_slices
	}

	#[must_use]
	pub const fn output_slices(&self) -> &'a [IoctlSlice] {
		self.output_slices
	}
}

pub struct IoctlRetryBuf {
	input_slices: [IoctlSlice; fuse_kernel::FUSE_IOCTL_MAX_IOV],
	output_slices: [IoctlSlice; fuse_kernel::FUSE_IOCTL_MAX_IOV],
	input_slices_len: usize,
	output_slices_len: usize,
}

impl IoctlRetryBuf {
	#[must_use]
	pub const fn new() -> IoctlRetryBuf {
		let zero = IoctlSlice::new(0, 0);
		Self {
			input_slices: [zero; fuse_kernel::FUSE_IOCTL_MAX_IOV],
			output_slices: [zero; fuse_kernel::FUSE_IOCTL_MAX_IOV],
			input_slices_len: 0,
			output_slices_len: 0,
		}
	}

	#[must_use]
	pub fn input_slices(&self) -> &[IoctlSlice] {
		&self.input_slices[..self.input_slices_len]
	}

	#[must_use]
	pub fn output_slices(&self) -> &[IoctlSlice] {
		&self.output_slices[..self.output_slices_len]
	}

	#[must_use]
	pub fn borrow(&self) -> IoctlRetry {
		IoctlRetry {
			input_slices: self.input_slices(),
			output_slices: self.output_slices(),
		}
	}

	fn check_add(&self) -> Result<(), IoctlRetryError> {
		let input_len = self.input_slices_len;
		let output_len = self.output_slices_len;
		if input_len + output_len + 1 > fuse_kernel::FUSE_IOCTL_MAX_IOV {
			return Err(IoctlRetryError::TooManySlices);
		}
		Ok(())
	}

	pub fn add_input_ptr<T>(
		&mut self,
		ptr: IoctlPtr<T>,
	) -> Result<(), IoctlRetryError> {
		self.check_add()?;
		let slice = IoctlSlice::new(ptr.addr(), size_of::<T>() as u64);
		self.input_slices[self.input_slices_len] = slice;
		self.input_slices_len += 1;
		Ok(())
	}

	pub fn add_input_slice(
		&mut self,
		slice: IoctlSlice,
	) -> Result<(), IoctlRetryError> {
		self.check_add()?;
		self.input_slices[self.input_slices_len] = slice;
		self.input_slices_len += 1;
		Ok(())
	}

	pub fn add_output_ptr<T>(
		&mut self,
		ptr: IoctlPtr<T>,
	) -> Result<(), IoctlRetryError> {
		self.check_add()?;
		let slice = IoctlSlice::new(ptr.addr(), size_of::<T>() as u64);
		self.output_slices[self.output_slices_len] = slice;
		self.output_slices_len += 1;
		Ok(())
	}

	pub fn add_output_slice(
		&mut self,
		slice: IoctlSlice,
	) -> Result<(), IoctlRetryError> {
		self.check_add()?;
		self.output_slices[self.output_slices_len] = slice;
		self.output_slices_len += 1;
		Ok(())
	}
}

// }}}

// IoctlRequestFlags {{{

/// Optional flags set on an [`IoctlRequest`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IoctlRequestFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IoctlRequestFlag {
	mask: u32,
}

mod request_flags {
	use crate::internal::fuse_kernel;
	bitflags!(IoctlRequestFlag, IoctlRequestFlags, u32, {
		IOCTL_COMPAT = fuse_kernel::FUSE_IOCTL_COMPAT;
		IOCTL_UNRESTRICTED = fuse_kernel::FUSE_IOCTL_UNRESTRICTED;
		IOCTL_32BIT = fuse_kernel::FUSE_IOCTL_32BIT;
		IOCTL_DIR = fuse_kernel::FUSE_IOCTL_DIR;
		IOCTL_COMPAT_X32 = fuse_kernel::FUSE_IOCTL_COMPAT_X32;
	});
}

// }}}
