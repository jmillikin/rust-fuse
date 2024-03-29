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

//! Implements the `FUSE_SETATTR` operation.

use core::fmt;
use core::time;

use crate::internal::fuse_kernel;
use crate::lock;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// SetattrRequest {{{

/// Request type for `FUSE_SETATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETATTR` operation.
pub struct SetattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: &'a fuse_kernel::fuse_setattr_in,
}

impl SetattrRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	fn get<T>(&self, bitmask: u32, value: T) -> Option<T> {
		if self.raw.valid & bitmask == 0 {
			return None;
		}
		Some(value)
	}

	#[must_use]
	fn get_timestamp(
		&self,
		bitmask: u32,
		seconds: u64,
		nanos: u32,
	) -> Option<crate::UnixTime> {
		if self.raw.valid & bitmask == 0 {
			return None;
		}
		Some(unsafe {
			crate::UnixTime::from_timespec_unchecked(seconds, nanos)
		})
	}

	#[must_use]
	pub fn handle(&self) -> Option<u64> {
		self.get(fuse_kernel::FATTR_FH, self.raw.fh)
	}

	#[must_use]
	pub fn size(&self) -> Option<u64> {
		self.get(fuse_kernel::FATTR_SIZE, self.raw.size)
	}

	#[must_use]
	pub fn lock_owner(&self) -> Option<lock::Owner> {
		self.get(
			fuse_kernel::FATTR_LOCKOWNER,
			lock::Owner::new(self.raw.lock_owner),
		)
	}

	#[must_use]
	pub fn atime(&self) -> Option<crate::UnixTime> {
		self.get_timestamp(
			fuse_kernel::FATTR_ATIME,
			self.raw.atime,
			self.raw.atimensec,
		)
	}

	#[must_use]
	pub fn atime_now(&self) -> bool {
		self.raw.valid & fuse_kernel::FATTR_ATIME_NOW > 0
	}

	#[must_use]
	pub fn mtime(&self) -> Option<crate::UnixTime> {
		self.get_timestamp(
			fuse_kernel::FATTR_MTIME,
			self.raw.mtime,
			self.raw.mtimensec,
		)
	}

	#[must_use]
	pub fn mtime_now(&self) -> bool {
		self.raw.valid & fuse_kernel::FATTR_MTIME_NOW > 0
	}

	#[must_use]
	pub fn ctime(&self) -> Option<crate::UnixTime> {
		self.get_timestamp(
			fuse_kernel::FATTR_CTIME,
			self.raw.ctime,
			self.raw.ctimensec,
		)
	}

	#[must_use]
	pub fn mode(&self) -> Option<node::Mode> {
		self.get(fuse_kernel::FATTR_MODE, node::Mode::new(self.raw.mode))
	}

	#[must_use]
	pub fn user_id(&self) -> Option<u32> {
		self.get(fuse_kernel::FATTR_UID, self.raw.uid)
	}

	#[must_use]
	pub fn group_id(&self) -> Option<u32> {
		self.get(fuse_kernel::FATTR_GID, self.raw.gid)
	}
}

impl server::sealed::Sealed for SetattrRequest<'_> {}

impl<'a> server::FuseRequest<'a> for SetattrRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_SETATTR)?;
		let header = dec.header();
		decode::node_id(header.nodeid)?;
		let raw: &fuse_kernel::fuse_setattr_in = dec.next_sized()?;

		if raw.valid & fuse_kernel::FATTR_ATIME > 0 {
			decode::check_timespec_nanos(raw.atimensec)?;
		}
		if raw.valid & fuse_kernel::FATTR_MTIME > 0 {
			decode::check_timespec_nanos(raw.mtimensec)?;
		}
		if raw.valid & fuse_kernel::FATTR_CTIME > 0 {
			decode::check_timespec_nanos(raw.ctimensec)?;
		}

		Ok(Self { header, raw })
	}
}

impl fmt::Debug for SetattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetattrRequest")
			.field("node_id", &self.node_id())
			.field("handle", &format_args!("{:?}", self.handle()))
			.field("size", &format_args!("{:?}", self.size()))
			.field("lock_owner", &format_args!("{:?}", self.lock_owner()))
			.field("atime", &format_args!("{:?}", self.atime()))
			.field("atime_now", &self.atime_now())
			.field("mtime", &format_args!("{:?}", self.mtime()))
			.field("mtime_now", &self.mtime_now())
			.field("ctime", &format_args!("{:?}", self.ctime()))
			.field("mode", &format_args!("{:?}", self.mode()))
			.field("user_id", &format_args!("{:?}", self.user_id()))
			.field("group_id", &format_args!("{:?}", self.group_id()))
			.finish()
	}
}

// }}}

// SetattrResponse {{{

/// Response type for `FUSE_SETATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_SETATTR` operation.
pub struct SetattrResponse {
	attr_out: node::FuseAttrOut,
}

impl SetattrResponse {
	#[inline]
	#[must_use]
	pub fn new(attributes: node::Attributes) -> SetattrResponse {
		Self {
			attr_out: node::FuseAttrOut::new(attributes),
		}
	}

	#[inline]
	#[must_use]
	pub fn attributes(&self) -> &node::Attributes {
		self.attr_out.attributes()
	}

	#[inline]
	#[must_use]
	pub fn attributes_mut(&mut self) -> &mut node::Attributes {
		self.attr_out.attributes_mut()
	}

	#[inline]
	#[must_use]
	pub fn cache_timeout(&self) -> time::Duration {
		self.attr_out.cache_timeout()
	}

	#[inline]
	pub fn set_cache_timeout(&mut self, timeout: time::Duration) {
		self.attr_out.set_cache_timeout(timeout)
	}
}

impl fmt::Debug for SetattrResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("SetattrResponse")
			.field("attributes", self.attributes())
			.field("cache_timeout", &self.cache_timeout())
			.finish()
	}
}

impl server::sealed::Sealed for SetattrResponse {}

impl server::FuseResponse for SetattrResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		if options.version_minor() >= 9 {
			return encode::sized(header, self.attr_out.as_v7p9());
		}
		encode::sized(header, self.attr_out.as_v7p1())
	}
}

// }}}
