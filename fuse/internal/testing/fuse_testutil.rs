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

pub const VERSION: (u32, u32) = (
	fuse_kernel::FUSE_KERNEL_VERSION,
	fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
);

pub struct MessageBuilder {
	header: Option<fuse_kernel::fuse_in_header>,
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
					+ size_of::<fuse_kernel::fuse_in_header>()) as u32;
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

	pub fn set_opcode(self, opcode: fuse_kernel::fuse_opcode) -> Self {
		self.set_header(|h| {
			h.opcode = opcode;
		})
	}

	pub fn set_header<HeaderFn>(mut self, header_fn: HeaderFn) -> Self
	where
		HeaderFn: FnOnce(&mut fuse_kernel::fuse_in_header),
	{
		let mut header = match self.header {
			None => fuse_kernel::fuse_in_header {
				len: 0,
				opcode: fuse_kernel::fuse_opcode(0),
				unique: 0xAABBCCDD,
				nodeid: 0,
				uid: 0,
				gid: 0,
				pid: 0,
				padding: 0,
			},
			Some(h) => h,
		};
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
		use fuse::operations::fuse_init;
		use fuse::server;
		use fuse::server::FuseRequest;
		use fuse::server::FuseRequestOptions;

		use $crate::DecodeRequestOpts;

		let opts = $crate::decode_request_opts!($opts);
		let request_len = $buf.as_slice().len();

		let mut init = fuse_init::FuseInitResponse::new();
		init.set_version(fuse::Version::new(
			opts.protocol_version.0,
			opts.protocol_version.1,
		));
		let req_opts = FuseRequestOptions::from_init_response(&init);

		let req_buf = $buf.as_aligned_slice().truncate(request_len);
		let request = server::Request::new(req_buf).unwrap();
		<$t>::from_request(request, req_opts).unwrap()
	}};
}

pub struct DecodeRequestOpts {
	pub protocol_version: (u32, u32),
}

#[macro_export]
macro_rules! decode_request_opts {
	({}) => {
		DecodeRequestOpts {
			protocol_version: $crate::VERSION,
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

#[macro_export]
macro_rules! encode_response {
	($response:expr) => {
		$crate::encode_response!($response, {})
	};
	($response:expr, $opts:tt $(,)?) => {{
		use fuse::operations::fuse_init;
		use fuse::server::FuseRequest;
		use fuse::server::FuseResponse;
		use fuse::server::FuseResponseOptions;
		use $crate::EncodeRequestOpts;
		use $crate::SendBufToVec;

		let opts = $crate::encode_request_opts!($opts);

		let mut init = fuse_init::FuseInitResponse::new();
		init.set_version(fuse::Version::new(
			opts.protocol_version.0,
			opts.protocol_version.1,
		));
		let resp_opts = FuseResponseOptions::from_init_response(&init);

		let request_id = core::num::NonZeroU64::new(0xAABBCCDD).unwrap();
		let mut resp_header = fuse::ResponseHeader::new(request_id);
		let response = $response.to_response(&mut resp_header, resp_opts);
		fuse::io::SendBuf::from(response).to_vec()
	}};
}

pub struct EncodeRequestOpts {
	pub protocol_version: (u32, u32),
}

#[macro_export]
macro_rules! encode_request_opts {
	({}) => {
		EncodeRequestOpts {
			protocol_version: $crate::VERSION,
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
