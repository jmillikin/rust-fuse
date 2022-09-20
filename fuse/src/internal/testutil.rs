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

#![allow(dead_code)]
#![allow(unused_macros)]

use std::cell::RefCell;
use std::mem::size_of;
use std::slice;

use crate::Version;
use crate::internal::fuse_kernel;
use crate::io;

pub(crate) struct MessageBuilder {
	header: Option<fuse_kernel::fuse_in_header>,
	buf: Vec<u8>,
}

impl MessageBuilder {
	pub(crate) fn new() -> Self {
		Self {
			header: None,
			buf: Vec::new(),
		}
	}

	pub(crate) fn build(self) -> Vec<u8> {
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

	pub(crate) fn build_aligned(self) -> io::ArrayBuffer {
		let buf = self.build();
		let mut out = io::ArrayBuffer::new();
		out.borrow_mut()[0..buf.len()].copy_from_slice(&buf);
		out
	}

	pub(crate) fn set_opcode(self, opcode: fuse_kernel::fuse_opcode) -> Self {
		self.set_header(|h| {
			h.opcode = opcode;
		})
	}

	pub(crate) fn set_header<HeaderFn>(mut self, header_fn: HeaderFn) -> Self
	where
		HeaderFn: FnOnce(&mut fuse_kernel::fuse_in_header),
	{
		let mut header = match self.header {
			None => fuse_kernel::fuse_in_header {
				len: 0,
				opcode: fuse_kernel::fuse_opcode(0),
				unique: 0,
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

	pub(crate) fn push_bytes(mut self, bytes: &[u8]) -> Self {
		self.buf.extend_from_slice(bytes);
		self
	}

	pub(crate) fn push_sized<T: Sized>(mut self, t: &T) -> Self {
		self.buf.extend_from_slice(unsafe {
			slice::from_raw_parts((t as *const T) as *const u8, size_of::<T>())
		});
		self
	}

	pub(crate) fn unpush(mut self, count: usize) -> Self {
		self.buf.truncate(self.buf.len() - count);
		self
	}
}

pub(crate) struct FakeSocket {
	pub(crate) write: RefCell<Option<Vec<u8>>>,
}

impl FakeSocket {
	pub(crate) fn new() -> Self {
		Self {
			write: RefCell::new(None),
		}
	}

	pub(crate) fn expect_write(&self) -> Vec<u8> {
		match &*self.write.borrow() {
			Some(w) => w.clone(),
			None => panic!("expected exactly one write to FakeSocket"),
		}
	}
}

impl io::ServerSocket for FakeSocket {
	type Error = std::io::Error;

	fn recv(
		&self,
		_buf: &mut [u8],
	) -> Result<usize, io::ServerRecvError<Self::Error>> {
		unimplemented!()
	}

	fn send(
		&self,
		buf: &[u8],
	) -> Result<(), io::ServerSendError<Self::Error>> {
		if self.write.borrow().is_some() {
			panic!("expected exactly one write to FakeSocket");
		}
		self.write.replace(Some(buf.into()));
		Ok(())
	}

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), io::ServerSendError<Self::Error>> {
		let mut vec = Vec::new();
		for buf in bufs {
			vec.extend(buf.to_vec());
		}
		self.send(&vec)
	}
}

macro_rules! decode_request {
	($t:ty, $buf: ident) => {
		decode_request!($t, $buf, {})
	};
	($t:ty, $buf: ident, $opts:tt $(,)?) => {{
		use crate::internal::testutil::DecodeRequestOpts;
		use crate::server::FuseRequestBuilder;

		let opts = decode_request_opts!($opts);
		let request_len = $buf.borrow().len();
		let fuse_request = FuseRequestBuilder::new()
			.version(opts.protocol_version())
			.build(&$buf.borrow()[..request_len])
			.unwrap();
		<$t>::from_fuse_request(&fuse_request).unwrap()
	}};
}

pub(crate) struct DecodeRequestOpts {
	pub(crate) protocol_version: (u32, u32),
}

impl DecodeRequestOpts {
	pub(crate) fn protocol_version(&self) -> Version {
		let (major, minor) = self.protocol_version;
		Version::new(major, minor)
	}
}

macro_rules! decode_request_opts {
	({}) => {
		DecodeRequestOpts {
			protocol_version: (
				fuse_kernel::FUSE_KERNEL_VERSION,
				fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
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

macro_rules! encode_response {
	($response:expr) => {
		encode_response!($response, {})
	};
	($response:expr, $opts:tt $(,)?) => {{
		use crate::internal::testutil::EncodeRequestOpts;
		use crate::io::encode::EncodeReply;

		let request_id = 0;
		let opts = encode_request_opts!($opts);
		let socket = crate::internal::testutil::FakeSocket::new();
		$response.encode(
			encode::SyncSendOnce::new(&socket),
			request_id,
			opts.protocol_version().minor(),
		).unwrap();
		socket.expect_write()
	}};
}

pub(crate) struct EncodeRequestOpts {
	pub(crate) protocol_version: (u32, u32),
}

impl EncodeRequestOpts {
	pub(crate) fn protocol_version(&self) -> Version {
		let (major, minor) = self.protocol_version;
		Version::new(major, minor)
	}
}

macro_rules! encode_request_opts {
	({}) => {
		EncodeRequestOpts {
			protocol_version: (
				fuse_kernel::FUSE_KERNEL_VERSION,
				fuse_kernel::FUSE_KERNEL_MINOR_VERSION,
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
