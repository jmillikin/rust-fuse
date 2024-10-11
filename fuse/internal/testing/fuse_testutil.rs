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

use std::mem::size_of;
use std::slice;

use fuse::kernel;
use fuse::server::{
	CuseSocket,
	FuseSocket,
	RecvError,
	SendError,
	Socket,
};

pub struct MessageBuilder {
	header: Option<kernel::fuse_in_header>,
	buf: Vec<u8>,
}

impl MessageBuilder {
	pub fn new() -> Self {
		Self {
			header: None,
			buf: Vec::new(),
		}
	}

	pub fn build(self) -> Vec<u8> {
		let mut out = match self.header {
			None => Vec::new(),
			Some(h) => {
				let mut h = h;
				h.len = (self.buf.len()
					+ size_of::<kernel::fuse_in_header>()) as u32;
				MessageBuilder::new().push_sized(&h).build()
			},
		};
		out.extend(self.buf.clone());
		out
	}

	pub fn build_aligned(self) -> fuse::io::MinReadBuffer {
		let buf = self.build();
		let mut out = fuse::io::MinReadBuffer::new();
		out.as_slice_mut()[..buf.len()].copy_from_slice(&buf);
		out
	}

	pub fn set_opcode(self, opcode: kernel::fuse_opcode) -> Self {
		self.set_header(|h| {
			h.opcode = opcode;
		})
	}

	pub fn set_header<HeaderFn>(mut self, header_fn: HeaderFn) -> Self
	where
		HeaderFn: FnOnce(&mut kernel::fuse_in_header),
	{
		const DEFAULT_HEADER: kernel::fuse_in_header = {
			let mut h = kernel::fuse_in_header::new();
			h.unique = 0xAABBCCDD;
			h
		};
		let mut header = self.header.unwrap_or(DEFAULT_HEADER);
		header_fn(&mut header);
		self.header = Some(header);
		self
	}

	pub fn push_bytes(mut self, bytes: &[u8]) -> Self {
		self.buf.extend_from_slice(bytes);
		self
	}

	pub fn push_sized<T: Sized>(mut self, t: &T) -> Self {
		self.buf.extend_from_slice(unsafe {
			slice::from_raw_parts((t as *const T) as *const u8, size_of::<T>())
		});
		self
	}

	pub fn unpush(mut self, count: usize) -> Self {
		self.buf.truncate(self.buf.len() - count);
		self
	}
}

#[macro_export]
macro_rules! decode_request {
	($t:ty, $buf: ident) => {
		$crate::decode_request!($t, $buf, {})
	};
	($t:ty, $buf: ident, $opts:tt $(,)?) => {{
		use fuse::server;
		use fuse::server::{FuseLayout, FuseRequest};

		use $crate::DecodeRequestOpts;

		let opts = $crate::decode_request_opts!($opts);
		let request_len = $buf.as_slice().len();

		let mut fuse_init_out = fuse::kernel::fuse_init_out::new();
		fuse_init_out.major = opts.protocol_version.0;
		fuse_init_out.minor = opts.protocol_version.1;
		let layout = FuseLayout::new(&fuse_init_out).unwrap();

		let req_buf = $buf.as_aligned_slice().truncate(request_len);
		let request = server::FuseRequest::new(req_buf, layout).unwrap();
		<$t>::try_from(request).unwrap()
	}};
}

pub struct DecodeRequestOpts {
	pub protocol_version: (u32, u32),
}

#[macro_export]
macro_rules! decode_request_opts {
	({}) => {
		DecodeRequestOpts {
			protocol_version: (
				fuse::kernel::FUSE_KERNEL_VERSION,
				fuse::kernel::FUSE_KERNEL_MINOR_VERSION,
			),
		}
	};
	({
		protocol_version: $version:expr,
	}) => {
		DecodeRequestOpts {
			protocol_version: $version,
		}
	};
}

pub trait SendBufToVec {
	fn to_vec(&self) -> Vec<u8>;
}

impl SendBufToVec for fuse::io::SendBuf<'_> {
	fn to_vec(&self) -> Vec<u8> {
		let mut vec = Vec::new();
		vec.reserve_exact(self.len());
		for chunk in self.chunks() {
			vec.extend_from_slice(chunk);
		}
		vec
	}
}

pub struct FakeSocket(std::cell::Cell<Vec<u8>>);

impl FakeSocket {
	pub fn new() -> FakeSocket {
		Self(std::cell::Cell::new(Vec::new()))
	}

	pub fn into_vec(self) -> Vec<u8> {
		self.0.into_inner()
	}
}

impl Socket for FakeSocket {
	type Error = ();
	fn recv(&self, _buf: &mut [u8]) -> Result<usize, RecvError<()>> {
		unimplemented!()
	}

	fn send(&self, buf: fuse::io::SendBuf) -> Result<(), SendError<()>> {
		self.0.set(buf.to_vec());
		Ok(())
	}
}

impl CuseSocket for FakeSocket {}

impl FuseSocket for FakeSocket {}

#[macro_export]
macro_rules! encode_response {
	($reply:expr) => {
		$crate::encode_response!($reply, {})
	};
	($reply:expr, $opts:tt $(,)?) => {{
		use fuse::server::{FuseLayout, FuseReplySender};
		use $crate::EncodeRequestOpts;

		let opts = $crate::encode_request_opts!($opts);

		let mut fuse_init_out = fuse::kernel::fuse_init_out::new();
		fuse_init_out.major = opts.protocol_version.0;
		fuse_init_out.minor = opts.protocol_version.1;
		let layout = FuseLayout::new(&fuse_init_out).unwrap();

		let request_id = core::num::NonZeroU64::new(0xAABBCCDD).unwrap();
		let socket = $crate::FakeSocket::new();
		FuseReplySender::new(socket, request_id, layout).ok($reply).unwrap();
		socket.into_vec()
	}};
}

pub struct EncodeRequestOpts {
	pub protocol_version: (u32, u32),
}

#[macro_export]
macro_rules! encode_request_opts {
	({}) => {
		EncodeRequestOpts {
			protocol_version: (
				fuse::kernel::FUSE_KERNEL_VERSION,
				fuse::kernel::FUSE_KERNEL_MINOR_VERSION,
			),
		}
	};
	({
		protocol_version: $version:expr,
	}) => {
		EncodeRequestOpts {
			protocol_version: $version,
		}
	};
}

#[macro_export]
macro_rules! build_request {
	($buf:ident, $t:ty, { $( . $step_fn:ident $step:tt )+ }) => {{
		let mut builder = $crate::MessageBuilder::new();
		$(
			builder = builder.$step_fn $step;
		)+
		$buf = builder.build_aligned();
		$crate::decode_request!($t, $buf)
	}};
}

#[macro_export]
macro_rules! new {
	($t:ty { $( $field:ident : $value:expr , )+ }) => {{
		let mut value = <$t>::new();
		$(
			value.$field = $value;
		)+
		value
	}}
}
