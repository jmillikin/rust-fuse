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
use core::ptr;

use crate::kernel;
use crate::server;
use crate::server::decode;

// ListxattrRequest {{{

/// Request type for `FUSE_LISTXATTR`.
pub struct ListxattrRequest<'a> {
	header: &'a kernel::fuse_in_header,
	body: &'a kernel::fuse_getxattr_in,
}

impl ListxattrRequest<'_> {
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.header.nodeid) }
	}

	#[inline]
	#[must_use]
	pub fn size(&self) -> Option<num::NonZeroUsize> {
		let size = usize::try_from(self.body.size).unwrap_or(usize::MAX);
		num::NonZeroUsize::new(size)
	}
}

try_from_fuse_request!(ListxattrRequest<'a>, |request| {
	let mut dec = request.decoder();
	dec.expect_opcode(kernel::fuse_opcode::FUSE_LISTXATTR)?;

	let header = dec.header();
	decode::node_id(header.nodeid)?;

	let body = dec.next_sized()?;
	Ok(Self { header, body })
});

impl fmt::Debug for ListxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ListxattrRequest")
			.field("node_id", &self.node_id())
			.field("size", &format_args!("{:?}", &self.size()))
			.finish()
	}
}

// }}}

// ListxattrNames {{{

#[derive(Copy, Clone)]
pub struct ListxattrNames<'a> {
	buf: &'a [u8],
}

impl<'a> ListxattrNames<'a> {
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &'a [u8] {
		self.buf
	}
}

impl fmt::Debug for ListxattrNames<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_list()
			.entries(XattrNamesIter(self.buf))
			.finish()
	}
}

impl server::FuseReply for ListxattrNames<'_> {
	fn send_to<S: server::FuseSocket>(
		&self,
		reply_sender: server::FuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		reply_sender.inner.send_1(self.buf)
	}
}

/// }}}

// XattrNamesIter {{{

struct XattrNamesIter<'a>(&'a [u8]);

impl<'a> core::iter::Iterator for XattrNamesIter<'a> {
	type Item = &'a crate::XattrName;

	fn next(&mut self) -> Option<&'a crate::XattrName> {
		if self.0.is_empty() {
			return None;
		}
		for (ii, byte) in self.0.iter().enumerate() {
			if *byte == 0 {
				let (name, _) = self.0.split_at(ii);
				let (_, next) = self.0.split_at(ii + 1);
				self.0 = next;
				return Some(unsafe {
					crate::XattrName::from_bytes_unchecked(name)
				});
			}
		}
		let name = unsafe { crate::XattrName::from_bytes_unchecked(self.0) };
		self.0 = &[];
		Some(name)
	}
}

// }}}

// ListxattrNamesWriter {{{

#[derive(Debug)]
#[non_exhaustive]
pub struct ListxattrCapacityError {}

pub struct ListxattrNamesWriter<'a> {
	buf: &'a mut [u8],
	position: usize,
}

impl<'a> ListxattrNamesWriter<'a> {
	#[inline]
	#[must_use]
	pub fn new(mut buf: &'a mut [u8]) -> ListxattrNamesWriter<'a> {
		if let Some(max_size) = crate::os::XATTR_LIST_MAX {
			if buf.len() > max_size {
				buf = &mut buf[..max_size];
			}
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
	pub fn into_names(self) -> ListxattrNames<'a> {
		ListxattrNames {
			buf: unsafe { self.buf.get_unchecked(..self.position) },
		}
	}

	pub fn try_push(
		&mut self,
		name: &crate::XattrName,
	) -> Result<(), ListxattrCapacityError> {
		let remaining_capacity = self.capacity() - self.position();
		if name.size() > remaining_capacity {
			return Err(ListxattrCapacityError {});
		}

		let name_start = self.position;
		self.position += name.size();

		let name_bytes = name.as_bytes();
		unsafe {
			let dst = self.buf.get_unchecked_mut(name_start..self.position);
			ptr::copy_nonoverlapping(
				name_bytes.as_ptr(),
				dst.as_mut_ptr(),
				name_bytes.len(),
			);
			*dst.get_unchecked_mut(name_bytes.len()) = 0;
		};
		Ok(())
	}
}

impl<'a> From<ListxattrNamesWriter<'a>> for ListxattrNames<'a> {
	#[inline]
	fn from(w: ListxattrNamesWriter<'a>) -> ListxattrNames<'a> {
		w.into_names()
	}
}

// }}}
