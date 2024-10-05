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

//! Implements the `FUSE_LOOKUP` operation.

use core::fmt;
use core::time;

use crate::internal::fuse_kernel;
use crate::internal::timestamp;
use crate::server;
use crate::server::decode;
use crate::server::encode;

// LookupRequest {{{

/// Request type for `FUSE_LOOKUP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LOOKUP` operation.
#[derive(Debug)]
pub struct LookupRequest<'a> {
	parent_id: crate::NodeId,
	name: &'a crate::NodeName,
}

impl LookupRequest<'_> {
	#[must_use]
	pub fn parent_id(&self) -> crate::NodeId {
		self.parent_id
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
		self.name
	}
}

impl server::sealed::Sealed for LookupRequest<'_> {}

impl<'a> server::FuseRequest<'a> for LookupRequest<'a> {
	fn from_request(
		request: server::Request<'a>,
		_options: server::FuseRequestOptions,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LOOKUP)?;
		Ok(Self {
			parent_id: decode::node_id(dec.header().nodeid)?,
			name: dec.next_node_name()?,
		})
	}
}

// }}}

// LookupResponse {{{

/// Response type for `FUSE_LOOKUP`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LOOKUP` operation.
pub struct LookupResponse {
	entry_out: fuse_kernel::fuse_entry_out,
}

impl LookupResponse {
	#[inline]
	#[must_use]
	pub fn new(entry: Option<crate::Entry>) -> LookupResponse {
		Self {
			entry_out: match entry {
				Some(entry) => entry.into_entry_out(),
				None => fuse_kernel::fuse_entry_out::zeroed(),
			},
		}
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> Option<&crate::Entry> {
		if self.entry_out.nodeid == 0 {
			return None;
		}
		Some(unsafe {
			crate::Entry::from_ref(&self.entry_out)
		})
	}

	#[inline]
	#[must_use]
	pub fn entry_mut(&mut self) -> Option<&mut crate::Entry> {
		if self.entry_out.nodeid == 0 {
			return None;
		}
		Some(unsafe {
			crate::Entry::from_ref_mut(&mut self.entry_out)
		})
	}

	#[inline]
	#[must_use]
	pub fn cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(
			self.entry_out.entry_valid,
			self.entry_out.entry_valid_nsec,
		)
	}

	#[inline]
	pub fn set_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.entry_out.entry_valid = seconds;
		self.entry_out.entry_valid_nsec = nanos;
	}

	#[inline]
	#[must_use]
	pub fn attribute_cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(
			self.entry_out.attr_valid,
			self.entry_out.attr_valid_nsec,
		)
	}

	#[inline]
	pub fn set_attribute_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.entry_out.attr_valid = seconds;
		self.entry_out.attr_valid_nsec = nanos;
	}
}

impl fmt::Debug for LookupResponse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("LookupResponse")
			.field("entry", &self.entry())
			.field("cache_timeout", &self.cache_timeout())
			.field("attribute_cache_timeout", &self.attribute_cache_timeout())
			.finish()
	}
}

impl server::sealed::Sealed for LookupResponse {}

impl server::FuseResponse for LookupResponse {
	fn to_response<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
		options: server::FuseResponseOptions,
	) -> server::Response<'a> {
		// In early versions of FUSE, `fuse_entry_out::nodeid` was a required
		// field and must be non-zero. FUSE v7.4 relaxed this so that a zero
		// node ID was the same as returning ENOENT, but with a cache hint.
		if self.entry_out.nodeid == 0 && options.version_minor() < 4 {
			return encode::error(header, crate::Error::NOT_FOUND);
		}
		let entry = unsafe { crate::Entry::from_ref(&self.entry_out) };
		if options.version_minor() >= 9 {
			return encode::sized(header, entry.as_v7p9());
		}
		encode::sized(header, entry.as_v7p1())
	}
}

// }}}
