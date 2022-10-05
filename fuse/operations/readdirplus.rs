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

//! Implements the `FUSE_READDIRPLUS` operation.

use core::convert::TryFrom;
use core::fmt;
use core::num;

use crate::internal::compat;
use crate::internal::dirent;
use crate::internal::fuse_kernel;
use crate::node;
use crate::server;
use crate::server::decode;
use crate::server::encode;

use crate::protocol::common::DebugHexU32;

// ReaddirplusRequest {{{

/// Request type for `FUSE_READDIRPLUS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READDIRPLUS` operation.
pub struct ReaddirplusRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_read_in<'a>>,
}

impl ReaddirplusRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn size(&self) -> usize {
		usize::try_from(self.body.as_v7p1().size).unwrap_or(usize::MAX)
	}

	#[must_use]
	pub fn offset(&self) -> Option<num::NonZeroU64> {
		num::NonZeroU64::new(self.body.as_v7p1().offset)
	}

	/// The value passed to [`OpendirResponse::set_handle`], or zero if not set.
	///
	/// [`OpendirResponse::set_handle`]: crate::operations::opendir::OpendirResponse::set_handle
	#[must_use]
	pub fn handle(&self) -> u64 {
		self.body.as_v7p1().fh
	}

	#[must_use]
	pub fn open_flags(&self) -> crate::OpenFlags {
		if let Some(body) = self.body.as_v7p9() {
			return body.flags;
		}
		0
	}
}

request_try_from! { ReaddirplusRequest : fuse }

impl decode::Sealed for ReaddirplusRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for ReaddirplusRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let version_minor = request.version_minor;
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_READDIRPLUS)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = if version_minor >= 9 {
			let body_v7p9 = dec.next_sized()?;
			compat::Versioned::new_read_v7p9(version_minor, body_v7p9)
		} else {
			let body_v7p1 = dec.next_sized()?;
			compat::Versioned::new_read_v7p1(version_minor, body_v7p1)
		};

		Ok(Self { header, body })
	}
}

impl fmt::Debug for ReaddirplusRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirplusRequest")
			.field("node_id", &self.node_id())
			.field("size", &self.size())
			.field("offset", &format_args!("{:?}", self.offset()))
			.field("handle", &self.handle())
			.field("open_flags", &DebugHexU32(self.open_flags()))
			.finish()
	}
}

// }}}

// ReaddirplusResponse {{{

/// Response type for `FUSE_READDIRPLUS`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_READDIRPLUS` operation.
pub struct ReaddirplusResponse<'a> {
	entries: ReaddirplusEntries<'a>,
}

impl ReaddirplusResponse<'_> {
	/// An empty `ReaddirplusResponse`, containing no entries.
	///
	/// This is useful for returning end-of-stream responses.
	pub const EMPTY: &'static ReaddirplusResponse<'static> = &ReaddirplusResponse {
		entries: ReaddirplusEntries { buf: b"" },
	};
}

impl<'a> ReaddirplusResponse<'a> {
	#[inline]
	#[must_use]
	pub fn new(entries: ReaddirplusEntries<'a>) -> ReaddirplusResponse<'a> {
		Self { entries }
	}

	#[inline]
	pub fn entries(&self) -> impl Iterator<Item = ReaddirplusEntry> {
		ReaddirplusEntriesIter::new(&self.entries)
	}
}

impl fmt::Debug for ReaddirplusResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirplusResponse")
			.field("entries", &self.entries)
			.finish()
	}
}

response_send_funcs!(ReaddirplusResponse<'_>);

impl ReaddirplusResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		if self.entries.is_empty() {
			enc.encode_header_only()
		} else {
			enc.encode_bytes(self.entries.buf)
		}
	}
}

// }}}

// ReaddirEntry {{{

#[derive(Clone, Copy)]
pub struct ReaddirplusEntry<'a> {
	dirent: fuse_kernel::fuse_direntplus,
	name: &'a node::Name,
}

impl<'a> ReaddirplusEntry<'a> {
	#[inline]
	#[must_use]
	pub fn new(
		node_id: node::Id,
		name: &'a node::Name,
		offset: num::NonZeroU64,
	) -> ReaddirplusEntry<'a> {
		Self {
			dirent: fuse_kernel::fuse_direntplus {
				dirent: fuse_kernel::fuse_dirent {
					ino: node_id.get(),
					off: offset.get(),
					namelen: name.as_bytes().len() as u32,
					..fuse_kernel::fuse_dirent::zeroed()
				},
				entry_out: fuse_kernel::fuse_entry_out {
					nodeid: node_id.get(),
					..fuse_kernel::fuse_entry_out::zeroed()
				},
			},
			name,
		}
	}

	#[inline]
	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.dirent.dirent.ino) }
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}

	#[inline]
	#[must_use]
	pub fn offset(&self) -> num::NonZeroU64 {
		unsafe { num::NonZeroU64::new_unchecked(self.dirent.dirent.off) }
	}

	#[inline]
	#[must_use]
	pub fn file_type(&self) -> Option<node::Type> {
		node::Type::from_bits(self.dirent.dirent.r#type)
	}

	#[inline]
	pub fn set_file_type(&mut self, file_type: node::Type) {
		self.dirent.dirent.r#type = file_type.as_bits();
	}

	#[inline]
	#[must_use]
	pub fn node_attr(&self) -> &crate::NodeAttr {
		crate::NodeAttr::new_ref(&self.dirent.entry_out.attr)
	}

	#[inline]
	#[must_use]
	pub fn mut_node_attr(&mut self) -> &mut crate::NodeAttr {
		crate::NodeAttr::new_ref_mut(&mut self.dirent.entry_out.attr)
	}
}

impl fmt::Debug for ReaddirplusEntry<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirEntry")
			.field("node_id", &self.node_id())
			.field("offset", &self.offset())
			.field("file_type", &format_args!("{:?}", self.file_type()))
			.field("name", &self.name())
			.field("node_attr", &self.node_attr())
			.finish()
	}
}

// }}}

// ReaddirplusEntries {{{

#[derive(Copy, Clone)]
pub struct ReaddirplusEntries<'a> {
	buf: &'a [u8],
}

impl<'a> ReaddirplusEntries<'a> {
	#[inline]
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.buf.is_empty()
	}
}

impl fmt::Debug for ReaddirplusEntries<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_list()
			.entries(ReaddirplusEntriesIter::new(self))
			.finish()
	}
}

// }}}

// ReaddirplusEntriesWriter {{{

#[derive(Debug)]
#[non_exhaustive]
pub struct ReaddirplusCapacityError {}

pub struct ReaddirplusEntriesWriter<'a> {
	buf: &'a mut [u8],
	position: usize,
}

impl<'a> ReaddirplusEntriesWriter<'a> {
	#[inline]
	#[must_use]
	pub fn new(mut buf: &'a mut [u8]) -> ReaddirplusEntriesWriter<'a> {
		let max_len = usize::from(u16::MAX);
		if buf.len() > max_len {
			buf = &mut buf[..max_len];
		}
		Self { buf, position: 0 }
	}

	#[inline]
	#[must_use]
	pub fn capacity(&self) -> usize {
		self.buf.len()
	}

	#[inline]
	#[must_use]
	pub fn position(&self) -> usize {
		self.position
	}

	#[inline]
	#[must_use]
	pub fn into_entries(self) -> ReaddirplusEntries<'a> {
		ReaddirplusEntries {
			buf: unsafe { self.buf.get_unchecked(..self.position) },
		}
	}

	#[inline]
	#[must_use]
	pub fn entry_size(entry: &ReaddirplusEntry) -> usize {
		dirent::entry_size::<fuse_kernel::fuse_direntplus>(entry.name)
	}

	pub fn try_push(
		&mut self,
		entry: &ReaddirplusEntry,
	) -> Result<(), ReaddirplusCapacityError> {
		let remaining_capacity = self.capacity() - self.position();
		let entry_size = Self::entry_size(entry);
		if entry_size > remaining_capacity {
			return Err(ReaddirplusCapacityError {});
		}

		let entry_start = self.position;
		self.position += entry_size;

		unsafe {
			let dst = self.buf.get_unchecked_mut(entry_start..self.position);
			dirent::write_unchecked(entry.dirent, entry.name, dst);
		};
		Ok(())
	}
}

// }}}

// ReaddirplusEntriesIter {{{

struct ReaddirplusEntriesIter<'a> {
	buf: &'a [u8],
}

impl<'a> ReaddirplusEntriesIter<'a> {
	#[inline]
	#[must_use]
	fn new(entries: &ReaddirplusEntries<'a>) -> ReaddirplusEntriesIter<'a> {
		Self { buf: entries.buf }
	}
}

impl<'a> Iterator for ReaddirplusEntriesIter<'a> {
	type Item = ReaddirplusEntry<'a>;

	fn next(&mut self) -> Option<ReaddirplusEntry<'a>> {
		if self.buf.is_empty() {
			return None;
		}

		use fuse_kernel::fuse_direntplus as T;
		unsafe {
			let (dirent, name) = dirent::read_unchecked::<T>(self.buf);
			let entry_size = dirent::entry_size::<T>(name);
			self.buf = self.buf.get_unchecked(entry_size..);

			Some(ReaddirplusEntry { dirent, name })
		}
	}
}

// }}}
