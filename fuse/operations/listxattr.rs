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

//! Implements the `FUSE_LISTXATTR` operation.

use core::convert::TryFrom;
use core::fmt;
use core::num;
use core::ptr;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode;
use crate::server::encode;
use crate::xattr;

#[cfg(target_os = "freebsd")]
macro_rules! xattr_name_list_max_size {
	() => { None }
}

#[cfg(target_os = "linux")]
macro_rules! xattr_name_list_max_size {
	() => { Some(xattr::XATTR_LIST_MAX) }
}

const NAMES_LIST_MAX_SIZE: Option<usize> = xattr_name_list_max_size!();

// ListxattrRequest {{{

/// Request type for `FUSE_LISTXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LISTXATTR` operation.
pub struct ListxattrRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	body: &'a fuse_kernel::fuse_getxattr_in,
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

request_try_from! { ListxattrRequest : fuse }

impl decode::Sealed for ListxattrRequest<'_> {}

impl<'a> decode::FuseRequest<'a> for ListxattrRequest<'a> {
	fn from_fuse_request(
		request: &server::FuseRequest<'a>,
	) -> Result<Self, server::RequestError> {
		let mut dec = request.decoder();
		dec.expect_opcode(fuse_kernel::FUSE_LISTXATTR)?;

		let header = dec.header();
		decode::node_id(header.nodeid)?;

		let body = dec.next_sized()?;
		Ok(Self { header, body })
	}
}

impl fmt::Debug for ListxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ListxattrRequest")
			.field("node_id", &self.node_id())
			.field("size", &format_args!("{:?}", &self.size()))
			.finish()
	}
}

// }}}

// ListxattrResponse {{{

/// Response type for `FUSE_LISTXATTR`.
///
/// See the [module-level documentation](self) for an overview of the
/// `FUSE_LISTXATTR` operation.
pub struct ListxattrResponse<'a> {
	output: ListxattrOutput<'a>,
}

enum ListxattrOutput<'a> {
	Names(ListxattrNames<'a>),
	Size(usize),
}

impl<'a> ListxattrResponse<'a> {
	#[inline]
	#[must_use]
	pub fn with_names(
		names: impl Into<ListxattrNames<'a>>,
	) -> ListxattrResponse<'a> {
		ListxattrResponse {
			output: ListxattrOutput::Names(names.into()),
		}
	}

	#[inline]
	#[must_use]
	pub fn with_names_size(names_size: usize) -> ListxattrResponse<'a> {
		ListxattrResponse {
			output: ListxattrOutput::Size(names_size),
		}
	}
}

response_send_funcs!(ListxattrResponse<'_>);

impl fmt::Debug for ListxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut dbg = fmt.debug_struct("ListxattrResponse");
		match self.output {
			ListxattrOutput::Names(names) => {
				dbg.field("names", &names);
			},
			ListxattrOutput::Size(size) => {
				dbg.field("size", &size);
			},
		}
		dbg.finish()
	}
}

impl ListxattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		ctx: &server::ResponseContext,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, ctx.request_id);
		match self.output {
			ListxattrOutput::Names(names) => {
				enc.encode_bytes(names.buf)
			},
			ListxattrOutput::Size(size) => match check_list_size(size) {
				Ok(size_u32) => {
					enc.encode_sized(&fuse_kernel::fuse_getxattr_out {
						size: size_u32,
						padding: 0,
					})
				},
				Err(err) => enc.encode_error(err),
			},
		}
	}
}

#[inline]
fn check_list_size(list_size: usize) -> Result<u32, crate::Error> {
	if let Some(max_size) = NAMES_LIST_MAX_SIZE {
		if list_size > max_size {
			return Err(crate::Error::E2BIG);
		}
	}
	u32::try_from(list_size).map_err(|_| crate::Error::E2BIG)
}

// }}}

// ListxattrNames {{{

#[derive(Copy, Clone)]
pub struct ListxattrNames<'a> {
	buf: &'a [u8],
}

impl fmt::Debug for ListxattrNames<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_list()
			.entries(XattrNamesIter(self.buf))
			.finish()
	}
}

/// }}}

// XattrNamesIter {{{

struct XattrNamesIter<'a>(&'a [u8]);

impl<'a> core::iter::Iterator for XattrNamesIter<'a> {
	type Item = &'a xattr::Name;

	fn next(&mut self) -> Option<&'a xattr::Name> {
		if self.0.is_empty() {
			return None;
		}
		for (ii, byte) in self.0.iter().enumerate() {
			if *byte == 0 {
				let (name, _) = self.0.split_at(ii);
				let (_, next) = self.0.split_at(ii + 1);
				self.0 = next;
				return Some(unsafe { xattr::Name::from_bytes_unchecked(name) });
			}
		}
		let name = unsafe { xattr::Name::from_bytes_unchecked(self.0) };
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
		if let Some(max_size) = NAMES_LIST_MAX_SIZE {
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
		name: &xattr::Name,
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
