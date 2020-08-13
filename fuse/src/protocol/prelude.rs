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

use core::ascii;

pub(crate) use std::cmp::min;
#[allow(unused_imports)]
pub(crate) use std::ffi::{CStr, CString};
pub(crate) use std::marker::PhantomData;
pub(crate) use std::mem::size_of;
pub(crate) use std::time::Duration;
pub(crate) use std::{cmp, fmt, io, mem, ptr, slice, time};

pub(crate) use crate::internal::errors;
pub(crate) use crate::internal::fuse_io;
pub(crate) use crate::internal::fuse_kernel;

#[allow(unused_imports)]
pub(crate) use crate::internal::fuse_io::{
	DecodeRequest,
	EncodeResponse,
	RequestDecoder,
	ResponseEncoder,
};

pub(crate) fn try_node_id(raw: u64) -> io::Result<crate::NodeId> {
	match crate::NodeId::new(raw) {
		Some(x) => Ok(x),
		None => todo!("failure path in try_node_id"),
	}
}

pub(crate) struct DebugHexU32(pub(crate) u32);

impl fmt::Debug for DebugHexU32 {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{:#010X}", self.0)
	}
}

pub(crate) struct DebugBytesAsString<'a>(pub(crate) &'a [u8]);

impl fmt::Debug for DebugBytesAsString<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "\"")?;
		for byte in self.0.iter().flat_map(|&b| ascii::escape_default(b)) {
			fmt::Write::write_char(fmt, byte as char)?;
		}
		write!(fmt, "\"")
	}
}

pub(crate) struct DebugClosure<F>(pub(crate) F)
where
	F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Debug for DebugClosure<F>
where
	F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0(fmt)
	}
}
