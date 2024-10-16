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

use core::convert::TryFrom;
use core::fmt;
use core::num;

use crate::internal::compat;
use crate::internal::debug;
use crate::internal::dirent;
use crate::kernel;
use crate::server::decode;

// ReaddirplusRequest {{{

/// Request type for `FUSE_READDIRPLUS`.
#[derive(Clone, Copy)]
pub struct ReaddirplusRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: compat::Versioned<compat::fuse_read_in<'a>>,
}

impl ReaddirplusRequest<'_> {
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[must_use]
	pub fn size(&self) -> u32 {
		self.body.as_v7p1().size
	}

	#[must_use]
	pub fn offset(&self) -> Option<num::NonZeroU64> {
		num::NonZeroU64::new(self.body.as_v7p1().offset)
	}

	/// The value set in [`fuse_open_out::fh`], or zero if not set.
	///
	/// [`fuse_open_out::fh`]: crate::kernel::fuse_open_out::fh
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

try_from_fuse_request!(ReaddirplusRequest<'a>, |request| {
	let version_minor = request.layout.version_minor();
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_READDIRPLUS)?;

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
});

impl fmt::Debug for ReaddirplusRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirplusRequest")
			.field("node_id", &self.node_id())
			.field("size", &self.size())
			.field("offset", &format_args!("{:?}", self.offset()))
			.field("handle", &self.handle())
			.field("open_flags", &debug::hex_u32(self.open_flags()))
			.finish()
	}
}

// }}}

// ReaddirEntry {{{

#[derive(Clone, Copy)]
pub struct ReaddirplusEntry<'a> {
	dirent: kernel::fuse_direntplus,
	name: &'a crate::NodeName,
}

impl<'a> ReaddirplusEntry<'a> {
	#[inline]
	#[must_use]
	pub fn new(
		name: &'a crate::NodeName,
		offset: num::NonZeroU64,
		entry: crate::Entry,
	) -> ReaddirplusEntry<'a> {
		let node_id = entry.attributes().node_id();
		let mode = entry.attributes().mode();
		Self {
			dirent: kernel::fuse_direntplus {
				dirent: kernel::fuse_dirent {
					ino: node_id.get(),
					off: offset.get(),
					r#type: mode.type_bits(),
					namelen: name.as_bytes().len() as u32,
					..kernel::fuse_dirent::new()
				},
				entry_out: *entry.raw(),
			},
			name,
		}
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
		self.name
	}

	#[inline]
	#[must_use]
	pub fn offset(&self) -> num::NonZeroU64 {
		unsafe { num::NonZeroU64::new_unchecked(self.dirent.dirent.off) }
	}

	#[inline]
	#[must_use]
	pub fn entry(&self) -> &crate::Entry {
		unsafe { crate::Entry::from_ref(&self.dirent.entry_out) }
	}
}

impl fmt::Debug for ReaddirplusEntry<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirEntry")
			.field("name", &self.name())
			.field("offset", &self.offset())
			.field("entry", &self.entry())
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
	pub fn as_bytes(&self) -> &'a [u8] {
		self.buf
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
		dirent::entry_size::<kernel::fuse_direntplus>(entry.name)
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

		use kernel::fuse_direntplus as T;
		unsafe {
			let (dirent, name) = dirent::read_unchecked::<T>(self.buf);
			let entry_size = dirent::entry_size::<T>(name);
			self.buf = self.buf.get_unchecked(entry_size..);

			Some(ReaddirplusEntry { dirent, name })
		}
	}
}

// }}}
