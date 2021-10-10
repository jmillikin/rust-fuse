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
use core::mem::size_of;

use crate::error::ErrorCode;
use crate::internal::fuse_kernel;
use crate::protocol::common::NodeId;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct RequestHeader(fuse_kernel::fuse_in_header);

impl RequestHeader {
	pub(crate) fn new_ref(raw: &fuse_kernel::fuse_in_header) -> &Self {
		unsafe {
			&*(raw as *const fuse_kernel::fuse_in_header
				as *const RequestHeader)
		}
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

	pub fn size(&self) -> u32 {
		self.0.len
	}

	pub fn body_len(&self) -> u32 {
		const BODY_START: u32 = size_of::<fuse_kernel::fuse_in_header>() as u32;
		return self.0.len.saturating_sub(BODY_START);
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
			.field("size", &self.0.len)
			.field("body_len", &self.body_len())
			.finish()
	}
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ResponseHeader(fuse_kernel::fuse_out_header);

impl ResponseHeader {
	pub fn request_id(&self) -> u64 {
		self.0.unique
	}

	pub fn error(&self) -> Option<ErrorCode> {
		let code = core::num::NonZeroU16::new((-self.0.error) as u16)?;
		Some(ErrorCode::new(code))
	}

	pub fn error_name(&self) -> Option<&'static str> {
		let code = self.error()?;
		code.name()
	}

	pub fn size(&self) -> u32 {
		self.0.len
	}

	pub fn body_len(&self) -> u32 {
		const BODY_START: u32 =
			size_of::<fuse_kernel::fuse_out_header>() as u32;
		return self.0.len.saturating_sub(BODY_START);
	}
}

impl fmt::Debug for ResponseHeader {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ResponseHeader")
			.field("request_id", &self.0.unique)
			.field("error", &format_args!("{:?}", &self.error()))
			.field("error_name", &format_args!("{:?}", &self.error_name()))
			.field("size", &self.0.len)
			.field("body_len", &self.body_len())
			.finish()
	}
}
