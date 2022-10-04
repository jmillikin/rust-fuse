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

mod io {
	use fuse_kernel::FUSE_MIN_READ_BUFFER;

	pub struct ArrayBuffer(ArrayBufferImpl);

	#[repr(align(8))]
	struct ArrayBufferImpl([u8; FUSE_MIN_READ_BUFFER]);

	impl ArrayBuffer {
		pub fn new() -> Self {
			ArrayBuffer(ArrayBufferImpl([0u8; FUSE_MIN_READ_BUFFER]))
		}

		#[allow(unused)]
		pub fn borrow(&self) -> &[u8] {
			&self.0 .0
		}

		pub fn borrow_mut(&mut self) -> &mut [u8] {
			&mut self.0 .0
		}
	}
}

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

	pub fn build_aligned(self) -> io::ArrayBuffer {
		let buf = self.build();
		let mut out = io::ArrayBuffer::new();
		out.borrow_mut()[0..buf.len()].copy_from_slice(&buf);
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
macro_rules! struct_fake_socket {
	() => {
pub struct FakeSocket {
	pub write: core::cell::RefCell<Option<Vec<u8>>>,
}

impl FakeSocket {
	pub fn new() -> Self {
		Self {
			write: core::cell::RefCell::new(None),
		}
	}

	pub fn expect_write(&self) -> Vec<u8> {
		match &*self.write.borrow() {
			Some(w) => w.clone(),
			None => panic!("expected exactly one write to FakeSocket"),
		}
	}
}

impl fuse::server::io::Socket for FakeSocket {
	type Error = std::io::Error;

	fn recv(
		&self,
		_buf: &mut [u8],
	) -> Result<usize, fuse::server::io::RecvError<Self::Error>> {
		unimplemented!()
	}

	fn send(
		&self,
		buf: fuse::io::SendBuf,
	) -> Result<(), fuse::server::io::SendError<Self::Error>> {
		if self.write.borrow().is_some() {
			panic!("expected exactly one write to FakeSocket");
		}
		self.write.replace(Some(buf.to_vec()));
		Ok(())
	}
}

	};
}

#[macro_export]
macro_rules! decode_request {
	($t:ty, $buf: ident) => {
		$crate::decode_request!($t, $buf, {})
	};
	($t:ty, $buf: ident, $opts:tt $(,)?) => {{
		use $crate::DecodeRequestOpts;
		use fuse::server::FuseRequestBuilder;
		use fuse::server::decode::FuseRequest;

		let opts = $crate::decode_request_opts!($opts);
		let request_len = $buf.borrow().len();
		let protocol_version = fuse::Version::new(
			opts.protocol_version.0,
			opts.protocol_version.1,
		);
		let fuse_request = FuseRequestBuilder::new()
			.version(protocol_version)
			.build(&$buf.borrow()[..request_len])
			.unwrap();
		<$t>::from_fuse_request(&fuse_request).unwrap()
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

#[macro_export]
macro_rules! encode_response {
	($response:expr) => {
		$crate::encode_response!($response, {})
	};
	($response:expr, $opts:tt $(,)?) => {{
		use fuse_testutil::EncodeRequestOpts;

		$crate::struct_fake_socket! {}

		let opts = $crate::encode_request_opts!($opts);

		let request_buf = $crate::MessageBuilder::new()
			.set_header(|h| {})
			.build_aligned();

		let protocol_version = fuse::Version::new(
			opts.protocol_version.0,
			opts.protocol_version.1,
		);
		let response_ctx = fuse::server::FuseRequestBuilder::new()
			.version(protocol_version)
			.build(request_buf.borrow())
			.unwrap()
			.response_context();

		let socket = FakeSocket::new();
		$response.send(&socket, &response_ctx).unwrap();
		socket.expect_write()
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
