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

#![allow(unused_imports)]

pub(super) use core::cmp::min;
pub(super) use core::marker::PhantomData;
pub(super) use core::mem::size_of;
pub(super) use core::time::Duration;
pub(super) use core::{cmp, fmt, mem, num, ptr, slice};

#[cfg(feature = "std")]
pub(super) use std::ffi::{CStr, CString};
#[cfg(feature = "std")]
pub(super) use std::time;

pub(super) use crate::error::ErrorCode;
pub(super) use crate::internal::fuse_kernel;
pub(super) use crate::io;
pub(super) use crate::io::decode;
pub(super) use crate::io::encode;
pub(super) use crate::protocol::common::{
	DebugBytesAsString,
	DebugClosure,
	DebugHexU32,
	FileMode,
	FileType,
	Node,
	NodeAttr,
	NodeId,
	NodeName,
	XattrError,
	XattrName,
};

pub(crate) fn try_node_id(raw: u64) -> Result<NodeId, io::RequestError> {
	match NodeId::new(raw) {
		Some(x) => Ok(x),
		None => Err(io::RequestError::MissingNodeId),
	}
}
