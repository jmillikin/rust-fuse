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

pub(crate) use core::cmp::min;
pub(crate) use core::marker::PhantomData;
pub(crate) use core::mem::size_of;
pub(crate) use core::time::Duration;
pub(crate) use core::{cmp, fmt, mem, num, ptr, slice};

#[cfg(feature = "std")]
pub(crate) use std::ffi::{CStr, CString};
#[cfg(feature = "std")]
pub(crate) use std::time;

pub(crate) use crate::internal::fuse_kernel;
pub(crate) use crate::server::io;
pub(crate) use crate::server::io::decode;
pub(crate) use crate::server::io::encode;
pub(crate) use crate::server::io::RequestError;
pub(crate) use crate::protocol::common::{
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

pub(crate) use crate::server::{CuseRequest, FuseRequest};

pub(crate) fn try_node_id(raw: u64) -> Result<NodeId, io::RequestError> {
	match NodeId::new(raw) {
		Some(x) => Ok(x),
		None => Err(io::RequestError::MissingNodeId),
	}
}

macro_rules! response_send_funcs {
	() => {
		pub fn send<S: crate::server::io::Socket>(
			&self,
			socket: &S,
			response_ctx: &crate::server::ResponseContext,
		) -> Result<(), crate::server::io::SendError<S::Error>> {
			use crate::server::io::encode::SyncSendOnce;
			let send = SyncSendOnce::new(socket);
			self.encode(send, response_ctx)
		}

		pub async fn send_async<S: crate::server::io::AsyncSocket>(
			&self,
			socket: &S,
			response_ctx: &crate::server::ResponseContext,
		) -> Result<(), crate::server::io::SendError<S::Error>> {
			use crate::server::io::encode::AsyncSendOnce;
			let send = AsyncSendOnce::new(socket);
			self.encode(send, response_ctx).await
		}
	};
}
