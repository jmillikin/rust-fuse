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

use crate::protocol::prelude::*;

// IoctlRequest {{{

pub struct IoctlRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_ioctl_in,
	buf: &'a [u8],
}

impl IoctlRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}
	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn command(&self) -> u32 {
		self.raw.cmd
	}

	pub fn buf(&self) -> &[u8] {
		self.buf
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for IoctlRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		version_minor: u32,
	) -> Result<Self, io::DecodeError> {
		let header = buf.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_IOCTL);
		let mut dec = decode::RequestDecoder::new(buf);
		let raw: &'a fuse_kernel::fuse_ioctl_in = dec.next_sized()?;

		/* TODO
		if (raw.flags & fuse_kernel::FUSE_IOCTL_DIR) > 0 {
			if !r.init_flags().get(InitFlag::IOCTL_DIR) {
				return Err(ENOTTY);
			}
		}
		*/

		let buf = dec.next_bytes(raw.in_size)?;
		Ok(Self { header, raw, buf })
	}
}

// }}}

// IoctlResponse {{{

const PAGE_SIZE: usize = 4096;

enum OutBuf {
	OutArr([u8; 4096], usize),
	OutVec(Vec<u8>),
}

pub struct IoctlResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_ioctl_out,
	buf: OutBuf,
}

impl<'a> IoctlResponse<'a> {
	// TODO: fix construction API
	pub fn new(request: &IoctlRequest) -> IoctlResponse<'a> {
		let out_size = request.raw.out_size as usize;
		let buf: OutBuf;
		if out_size > PAGE_SIZE {
			buf = OutBuf::OutVec(Vec::with_capacity(out_size));
		} else {
			buf = OutBuf::OutArr([0; 4096], out_size)
		}
		Self {
			phantom: PhantomData,
			raw: Default::default(),
			buf: buf,
		}
	}

	pub fn set_result(&mut self, result: i32) {
		self.raw.result = result;
	}

	pub fn buf(&self) -> &[u8] {
		match self.buf {
			OutBuf::OutVec(ref b) => b,
			OutBuf::OutArr(ref arr, buf_size) => {
				let (buf, _) = arr.split_at(buf_size);
				buf
			},
		}
	}

	pub fn buf_mut(&mut self) -> &mut [u8] {
		match self.buf {
			OutBuf::OutVec(ref mut b) => b,
			OutBuf::OutArr(ref mut arr, buf_size) => {
				let (buf, _) = arr.split_at_mut(buf_size);
				buf
			},
		}
	}
}

impl fmt::Debug for IoctlResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("IoctlResponse")
			.field("result", &self.raw.result)
			.field("buf", &self.buf())
			.finish()
	}
}

impl fuse_io::EncodeResponse for IoctlResponse<'_> {
	fn encode_response<'a, S: io::OutputStream>(
		&'a self,
		enc: fuse_io::ResponseEncoder<S>,
	) -> Result<(), S::Error> {
		todo!()
		//w.append_sized(&self.raw);
		//w.append_bytes(self.buf())
	}
}

// }}}
