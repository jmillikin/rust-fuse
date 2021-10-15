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

use crate::internal::fuse_kernel::fuse_in_header;
use crate::io;
use crate::io::decode::RequestBuf;
use crate::NodeId;

pub trait Request<'a, T> {
	fn decode(raw: &T) -> Result<Self, io::RequestError>
	where
		Self: Sized;
}

#[derive(Copy, Clone)]
pub struct RequestHeader(fuse_in_header);

impl RequestHeader {
	pub(crate) fn new_ref<'a>(raw: &'a fuse_in_header) -> &'a RequestHeader {
		unsafe { transmute(raw) }
	}

	pub(crate) fn from_buf<'a>(buf: RequestBuf<'a>) -> &'a RequestHeader {
		unsafe { transmute(buf.header()) }
	}

	pub fn opcode(&self) -> u32 {
		self.0.opcode.0
	}

	pub fn request_id(&self) -> u64 {
		self.0.unique
	}

	pub fn node_id(&self) -> Option<NodeId> {
		NodeId::new(self.0.nodeid)
	}

	pub fn user_id(&self) -> u32 {
		self.0.uid
	}

	pub fn group_id(&self) -> u32 {
		self.0.gid
	}

	pub fn process_id(&self) -> u32 {
		self.0.pid
	}

	pub fn len(&self) -> u32 {
		self.0.len
	}
}

impl fmt::Debug for RequestHeader {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("RequestHeader")
			.field("opcode", &self.0.opcode.0)
			.field("request_id", &self.0.unique)
			.field("node_id", &format_args!("{:?}", self.node_id()))
			.field("user_id", &self.0.uid)
			.field("group_id", &self.0.gid)
			.field("process_id", &self.0.pid)
			.field("len", &self.0.len)
			.finish()
	}
}
