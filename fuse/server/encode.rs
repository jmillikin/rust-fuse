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

use core::convert::TryFrom;
use core::mem;
use core::num;

use crate::internal::fuse_kernel;
use crate::io::SendBuf;
use crate::server;

const HEADER_LEN: usize = mem::size_of::<fuse_kernel::fuse_out_header>();

#[inline]
#[must_use]
pub(crate) fn header_only<'a>(
	header: &'a crate::ResponseHeader,
) -> server::Response<'a> {
	server::Response::new(SendBuf::new_1(
		HEADER_LEN,
		sized_to_slice(header),
	))
}

#[inline]
#[must_use]
pub(crate) fn error<'a>(
	header: &'a mut crate::ResponseHeader,
	err: crate::Error,
) -> server::Response<'a> {
	header.set_error(err.raw_fuse_error_code());
	header_only(header)
}

#[inline]
#[must_use]
pub(crate) fn bytes<'a>(
	header: &'a mut crate::ResponseHeader,
	bytes: &'a [u8],
) -> server::Response<'a> {
	let payload_len = bytes.len();
	let response_len = match checked_response_len(payload_len) {
		Some(len) => len,
		None => return error(header, crate::Error::OVERFLOW),
	};

	header.set_response_len(response_len.as_u32());
	server::Response::new(SendBuf::new_2(
		response_len.as_usize(),
		sized_to_slice(header),
		bytes,
	))
}

#[inline]
#[must_use]
pub(crate) fn sized<'a, T: Sized>(
	header: &'a mut crate::ResponseHeader,
	t: &'a T,
) -> server::Response<'a> {
	let payload_len = mem::size_of::<T>();
	let response_len = match checked_response_len(payload_len) {
		Some(len) => len,
		None => return error(header, crate::Error::OVERFLOW),
	};

	header.set_response_len(response_len.as_u32());
	server::Response::new(SendBuf::new_2(
		response_len.as_usize(),
		sized_to_slice(header),
		sized_to_slice(t),
	))
}

#[inline]
#[must_use]
pub(crate) fn sized2<'a, T1: Sized, T2: Sized>(
	header: &'a mut crate::ResponseHeader,
	t_1: &'a T1,
	t_2: &'a T2,
) -> server::Response<'a> {
	let mut payload_len = mem::size_of::<T1>();
	payload_len = payload_len.saturating_add(mem::size_of::<T2>());
	let response_len = match checked_response_len(payload_len) {
		Some(len) => len,
		None => return error(header, crate::Error::OVERFLOW),
	};

	header.set_response_len(response_len.as_u32());
	server::Response::new(SendBuf::new_3(
		response_len.as_usize(),
		sized_to_slice(header),
		sized_to_slice(t_1),
		sized_to_slice(t_2),
	))
}

#[inline]
#[must_use]
pub(crate) fn sized_bytes<'a, T: Sized>(
	header: &'a mut crate::ResponseHeader,
	t: &'a T,
	bytes: &'a [u8],
) -> server::Response<'a> {
	let mut payload_len = mem::size_of::<T>();
	payload_len = payload_len.saturating_add(bytes.len());
	let response_len = match checked_response_len(payload_len) {
		Some(len) => len,
		None => return error(header, crate::Error::OVERFLOW),
	};

	header.set_response_len(response_len.as_u32());
	server::Response::new(SendBuf::new_3(
		response_len.as_usize(),
		sized_to_slice(header),
		sized_to_slice(t),
		bytes,
	))
}

#[inline]
#[must_use]
pub(crate) fn sized_bytes2<'a, T: Sized>(
	header: &'a mut crate::ResponseHeader,
	t: &'a T,
	bytes_1: &'a [u8],
	bytes_2: &'a [u8],
) -> server::Response<'a> {
	let mut payload_len = mem::size_of::<T>();
	payload_len = payload_len.saturating_add(bytes_1.len());
	payload_len = payload_len.saturating_add(bytes_2.len());
	let response_len = match checked_response_len(payload_len) {
		Some(len) => len,
		None => return error(header, crate::Error::OVERFLOW),
	};

	header.set_response_len(response_len.as_u32());
	server::Response::new(SendBuf::new_4(
		response_len.as_usize(),
		sized_to_slice(header),
		sized_to_slice(t),
		bytes_1,
		bytes_2,
	))
}

#[inline]
#[must_use]
pub(crate) fn sized_bytes3<'a, T: Sized>(
	header: &'a mut crate::ResponseHeader,
	t: &'a T,
	bytes_1: &'a [u8],
	bytes_2: &'a [u8],
	bytes_3: &'a [u8],
) -> server::Response<'a> {
	let mut payload_len = mem::size_of::<T>();
	payload_len = payload_len.saturating_add(bytes_1.len());
	payload_len = payload_len.saturating_add(bytes_2.len());
	payload_len = payload_len.saturating_add(bytes_3.len());
	let response_len = match checked_response_len(payload_len) {
		Some(len) => len,
		None => return error(header, crate::Error::OVERFLOW),
	};

	header.set_response_len(response_len.as_u32());
	server::Response::new(SendBuf::new_5(
		response_len.as_usize(),
		sized_to_slice(header),
		sized_to_slice(t),
		bytes_1,
		bytes_2,
		bytes_3,
	))
}

#[inline]
pub(crate) fn sized_to_slice<T>(t: &T) -> &[u8] {
	let t_ptr = (t as *const T).cast::<u8>();
	unsafe { core::slice::from_raw_parts(t_ptr, mem::size_of::<T>()) }
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseLen(usize);

impl ResponseLen {
	#[inline]
	fn as_usize(self) -> usize {
		self.0
	}

	#[inline]
	fn as_u32(self) -> num::NonZeroU32 {
		unsafe { num::NonZeroU32::new_unchecked(self.0 as u32) }
	}
}

#[inline]
#[must_use]
pub(crate) fn checked_response_len(payload_len: usize) -> Option<ResponseLen> {
	let header_len = mem::size_of::<fuse_kernel::fuse_out_header>();
	let response_len = header_len.checked_add(payload_len)?;
	u32::try_from(response_len).ok()?;
	Some(ResponseLen(response_len))
}
