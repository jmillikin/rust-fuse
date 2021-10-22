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

use core::cell::RefCell;

use crate::protocol::prelude::*;
use crate::server::RequestHeader;

pub struct UnknownRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: RefCell<UnknownBody<'a>>,
}

enum UnknownBody<'a> {
	Raw(decode::RequestBuf<'a>),
	Parsed(Result<&'a [u8], io::RequestError>),
}

impl<'a> UnknownRequest<'a> {
	pub(crate) fn new(buf: decode::RequestBuf<'a>) -> Self {
		Self {
			header: buf.header(),
			body: RefCell::new(UnknownBody::Raw(buf)),
		}
	}

	pub fn header(&self) -> &RequestHeader {
		RequestHeader::new_ref(&self.header)
	}

	pub fn body(&self) -> Result<&'a [u8], io::RequestError> {
		let mut result: Result<&'a [u8], io::RequestError> = Ok(&[]);
		self.body.replace_with(|body| match body {
			UnknownBody::Raw(buf) => {
				let body_offset =
					size_of::<fuse_kernel::fuse_in_header>() as u32;
				let mut dec = decode::RequestDecoder::new(*buf);
				result = dec.next_bytes(self.header.len - body_offset);
				UnknownBody::Parsed(result)
			},
			UnknownBody::Parsed(r) => {
				result = *r;
				UnknownBody::Parsed(*r)
			},
		});
		result
	}
}

impl fmt::Debug for UnknownRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("UnknownRequest")
			.field("header", &self.header())
			.field("body", &format_args!("{:?}", self.body()))
			.finish()
	}
}
