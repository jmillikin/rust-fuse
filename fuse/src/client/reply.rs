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

use core::fmt;
use core::mem::transmute;
use core::num::{NonZeroI32, NonZeroU64};

use crate::internal::fuse_kernel::fuse_out_header;
use crate::io;

pub trait Reply<'a, T> {
	fn decode(raw: &T) -> Result<Self, io::ReplyError>
	where
		Self: Sized;
}

#[derive(Copy, Clone)]
pub struct ReplyHeader(fuse_out_header);

impl ReplyHeader {
	#[allow(dead_code)]
	pub(crate) fn new_ref<'a>(raw: &'a fuse_out_header) -> &'a ReplyHeader {
		unsafe { transmute(raw) }
	}

	pub fn request_id(&self) -> Option<NonZeroU64> {
		NonZeroU64::new(self.0.unique)
	}

	pub fn error(&self) -> Option<NonZeroI32> {
		NonZeroI32::new(self.0.error)
	}

	pub fn len(&self) -> u32 {
		self.0.len
	}
}

impl fmt::Debug for ReplyHeader {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ResponseHeader")
			.field("request_id", &format_args!("{:?}", &self.request_id()))
			.field("error", &format_args!("{:?}", &self.error()))
			.field("len", &self.0.len)
			.finish()
	}
}
