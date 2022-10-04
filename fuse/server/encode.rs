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

use core::mem::size_of;

use crate::io::SendBuf;
use crate::internal::fuse_kernel;
use crate::server::io;

pub(crate) trait SendOnce {
	type Result;

	fn send(self, slices: SendBuf) -> Self::Result;
}

pub(crate) struct SyncSendOnce<'a, S>(&'a S);
pub(crate) struct AsyncSendOnce<'a, S>(&'a S);

impl<'a, S: io::Socket> SyncSendOnce<'a, S> {
	pub(crate) fn new(socket: &'a S) -> Self {
		Self(socket)
	}
}

impl<'a, S: io::AsyncSocket> AsyncSendOnce<'a, S> {
	pub(crate) fn new(socket: &'a S) -> Self {
		Self(socket)
	}
}

impl<S: io::Socket> SendOnce for SyncSendOnce<'_, S> {
	type Result = Result<(), io::SendError<S::Error>>;

	fn send(self, slices: SendBuf) -> Self::Result {
		self.0.send(slices)
	}
}

impl<S: io::AsyncSocket> SendOnce for AsyncSendOnce<'_, S> {
	type Result = S::SendFuture;

	fn send(self, slices: SendBuf) -> Self::Result {
		self.0.send(slices)
	}
}

pub(crate) struct ReplyEncoder<S> {
	sender: S,
	request_id: u64,
}

impl<S: SendOnce> ReplyEncoder<S> {
	pub(crate) fn new(sender: S, request_id: u64) -> Self {
		Self { sender, request_id }
	}

	pub(crate) fn encode_error(self, err: crate::Error) -> S::Result {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: err.raw_fuse_error_code(),
			unique: self.request_id,
		};
		let out_hdr_buf = sized_to_slice(&out_hdr);

		self.sender.send(SendBuf::new_1(len, out_hdr_buf))
	}

	pub(crate) fn encode_unsolicited<T: core::fmt::Debug>(
		self,
		notify_code: fuse_kernel::fuse_notify_code,
		body: &T,
		name_bytes: Option<&[u8]>,
	) -> S::Result {
		let mut payload_len = size_of::<T>();
		if let Some(name_bytes) = name_bytes {
			payload_len = payload_len.saturating_add(name_bytes.len());
			payload_len = payload_len.saturating_add(1);
		}

		let mut len = size_of::<fuse_kernel::fuse_out_header>();
		match len.checked_add(payload_len) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, payload_len),
		}

		if size_of::<usize>() > size_of::<u32>() {
			if len > u32::MAX as usize {
				panic!("{} overflows u32", len);
			}
		}

		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: notify_code.0 as i32,
			unique: 0,
		};
		let out_hdr_buf = sized_to_slice(&out_hdr);
		let body_buf = sized_to_slice(body);
		if let Some(name_bytes) = name_bytes {
			self.sender.send(SendBuf::new_4(
				payload_len,
				out_hdr_buf,
				body_buf,
				name_bytes,
				b"\0",
			))
		} else {
			self.sender.send(SendBuf::new_2(
				payload_len,
				out_hdr_buf,
				body_buf,
			))
		}
	}

	pub(crate) fn encode_sized<T: Sized>(self, t: &T) -> S::Result {
		self.encode_bytes(sized_to_slice(t))
	}

	pub(crate) fn encode_sized_bytes<T: Sized>(
		self,
		bytes_1: &[u8],
		t: &T,
	) -> S::Result {
		self.encode_bytes_2(bytes_1, sized_to_slice(t))
	}

	pub(crate) fn encode_sized_sized<T1: Sized, T2: Sized>(
		self,
		t_1: &T1,
		t_2: &T2,
	) -> S::Result {
		self.encode_bytes_2(sized_to_slice(t_1), sized_to_slice(t_2))
	}

	pub(crate) fn encode_header_only(self) -> S::Result {
		let len = size_of::<fuse_kernel::fuse_out_header>();
		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf = sized_to_slice(&out_hdr);

		self.sender.send(SendBuf::new_1(len, out_hdr_buf))
	}

	pub(crate) fn encode_bytes(self, bytes: &[u8]) -> S::Result {
		let mut len = size_of::<fuse_kernel::fuse_out_header>();

		match len.checked_add(bytes.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes.len()),
		}

		if size_of::<usize>() > size_of::<u32>() {
			if len > u32::MAX as usize {
				panic!("{} overflows u32", len);
			}
		}

		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf = sized_to_slice(&out_hdr);

		self.sender.send(SendBuf::new_2(len, out_hdr_buf, bytes))
	}

	pub(crate) fn encode_bytes_2(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
	) -> S::Result {
		let mut len = size_of::<fuse_kernel::fuse_out_header>();

		match len.checked_add(bytes_1.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_1.len()),
		}
		match len.checked_add(bytes_2.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_2.len()),
		}

		if size_of::<usize>() > size_of::<u32>() {
			if len > u32::MAX as usize {
				panic!("{} overflows u32", len);
			}
		}

		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf = sized_to_slice(&out_hdr);

		self.sender.send(SendBuf::new_3(len, out_hdr_buf, bytes_1, bytes_2))
	}

	pub(crate) fn encode_bytes_4(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
		bytes_3: &[u8],
		bytes_4: &[u8],
	) -> S::Result {
		let mut len = size_of::<fuse_kernel::fuse_out_header>();

		match len.checked_add(bytes_1.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_1.len()),
		}
		match len.checked_add(bytes_2.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_2.len()),
		}
		match len.checked_add(bytes_3.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_3.len()),
		}
		match len.checked_add(bytes_4.len()) {
			Some(x) => len = x,
			None => panic!("{} + {} overflows usize", len, bytes_4.len()),
		}

		if size_of::<usize>() > size_of::<u32>() {
			if len > u32::MAX as usize {
				panic!("{} overflows u32", len);
			}
		}

		let out_hdr = fuse_kernel::fuse_out_header {
			len: len as u32,
			error: 0,
			unique: self.request_id,
		};
		let out_hdr_buf = sized_to_slice(&out_hdr);

		self.sender.send(SendBuf::new_5(len, out_hdr_buf, bytes_1, bytes_2, bytes_3, bytes_4))
	}
}

#[inline]
fn sized_to_slice<T>(t: &T) -> &[u8] {
	let t_ptr = (t as *const T).cast::<u8>();
	unsafe { core::slice::from_raw_parts(t_ptr, size_of::<T>()) }
}
