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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "listxattr_test")]
mod listxattr_test;

// ListxattrRequest {{{

/// Request type for [`FuseHandlers::listxattr`].
///
/// [`FuseHandlers::listxattr`]: ../../trait.FuseHandlers.html#method.listxattr
pub struct ListxattrRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	size: Option<num::NonZeroU32>,
}

impl ListxattrRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn size(&self) -> Option<num::NonZeroU32> {
		self.size
	}
}

impl fmt::Debug for ListxattrRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ListxattrRequest")
			.field("node_id", &self.node_id)
			.field("size", &format_args!("{:?}", &self.size))
			.finish()
	}
}

impl<'a> decode::DecodeRequest<'a, decode::FUSE> for ListxattrRequest<'a> {
	fn decode(
		buf: decode::RequestBuf<'a>,
		_version_minor: u32,
	) -> Result<Self, io::RequestError> {
		buf.expect_opcode(fuse_kernel::FUSE_LISTXATTR)?;

		let mut dec = decode::RequestDecoder::new(buf);
		let raw: &fuse_kernel::fuse_getxattr_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(buf.header().nodeid)?,
			size: num::NonZeroU32::new(raw.size),
		})
	}
}

// }}}

// ListxattrResponse {{{

/// Response type for [`FuseHandlers::listxattr`].
///
/// [`FuseHandlers::listxattr`]: ../../trait.FuseHandlers.html#method.listxattr
pub struct ListxattrResponse<'a> {
	buf: ListxattrBuf<'a>,
}

impl<'a> ListxattrResponse<'a> {
	pub fn without_capacity() -> ListxattrResponse<'a> {
		Self {
			buf: ListxattrBuf::SizeOnly { size: 0 },
		}
	}

	#[cfg(feature = "std")]
	pub fn with_max_size(max_size: u32) -> ListxattrResponse<'a> {
		Self {
			buf: ListxattrBuf::Owned {
				cap: Vec::new(),
				max_size: max_size as usize,
			},
		}
	}

	pub fn with_capacity(capacity: &'a mut [u8]) -> ListxattrResponse<'a> {
		Self {
			buf: ListxattrBuf::Borrowed {
				cap: capacity,
				size: 0,
			},
		}
	}

	pub fn names(&self) -> impl Iterator<Item = &XattrName> {
		XattrNamesIter::new(&self.buf)
	}

	pub fn add_name(&mut self, name: &XattrName) {
		self.try_add_name(name).unwrap()
	}

	pub fn try_add_name(&mut self, name: &XattrName) -> Result<(), XattrError> {
		use crate::XATTR_LIST_MAX;

		let name = name.as_bytes();
		let name_len = name.len() as usize;
		let name_buf_len = name_len + 1; // includes NUL terminator

		let name_buf = match &mut self.buf {
			ListxattrBuf::SizeOnly { size } => {
				let new_size = *size as usize + name_buf_len;
				if new_size > XATTR_LIST_MAX {
					return Err(XattrError::exceeds_list_max(new_size));
				}
				*size += name_buf_len as u32;
				return Ok(());
			},
			ListxattrBuf::Borrowed {
				cap,
				size: size_ref,
			} => {
				let current_size = *size_ref;
				let new_size = current_size + name_buf_len;
				if new_size > cap.len() {
					return Err(XattrError::exceeds_capacity(
						new_size,
						cap.len(),
					));
				}
				if new_size > XATTR_LIST_MAX {
					return Err(XattrError::exceeds_list_max(new_size));
				}
				let (_, remaining_cap) = cap.split_at_mut(current_size);
				let (name_buf, _) = remaining_cap.split_at_mut(name_buf_len);
				*size_ref = new_size;
				name_buf
			},
			#[cfg(feature = "std")]
			ListxattrBuf::Owned { cap, max_size } => {
				let current_size = cap.len();
				let new_size = current_size + name_buf_len;
				if new_size > XATTR_LIST_MAX {
					return Err(XattrError::exceeds_list_max(new_size));
				}
				if new_size > *max_size {
					return Err(XattrError::exceeds_capacity(
						new_size, *max_size,
					));
				}
				cap.resize(new_size, 0u8);
				let (_, entry_buf) = cap.split_at_mut(current_size);
				entry_buf
			},
		};

		debug_assert!(name_buf.len() == name_buf_len);

		let (name_no_nul, name_nul) = name_buf.split_at_mut(name_len);
		name_no_nul.copy_from_slice(name);
		name_nul[0] = 0;
		Ok(())
	}
}

enum ListxattrBuf<'a> {
	SizeOnly {
		size: u32,
	},
	#[cfg(feature = "std")]
	Owned {
		cap: Vec<u8>,
		max_size: usize,
	},
	Borrowed {
		cap: &'a mut [u8],
		size: usize,
	},
}

impl fmt::Debug for ListxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut out = fmt.debug_struct("ListxattrResponse");
		let names = DebugClosure(|fmt| {
			fmt.debug_list()
				.entries(XattrNamesIter::new(&self.buf))
				.finish()
		});
		match &self.buf {
			ListxattrBuf::SizeOnly { size } => {
				out.field("size", size);
			},
			#[cfg(feature = "std")]
			ListxattrBuf::Owned { .. } => {
				out.field("names", &names);
			},
			ListxattrBuf::Borrowed { .. } => {
				out.field("names", &names);
			},
		}
		out.finish()
	}
}

struct XattrNamesIter<'a>(&'a [u8]);

impl<'a> XattrNamesIter<'a> {
	fn new(buf: &'a ListxattrBuf) -> XattrNamesIter<'a> {
		XattrNamesIter(match buf {
			ListxattrBuf::SizeOnly { .. } => &[],
			#[cfg(feature = "std")]
			ListxattrBuf::Owned { cap, .. } => cap.as_ref(),
			ListxattrBuf::Borrowed { cap, size } => {
				let (bytes, _) = cap.split_at(*size);
				bytes
			},
		})
	}
}

impl<'a> core::iter::Iterator for XattrNamesIter<'a> {
	type Item = &'a XattrName;

	fn next(&mut self) -> Option<&'a XattrName> {
		let len = self.0.len();
		if len == 0 {
			return None;
		}
		for ii in 0..len {
			if self.0[ii] == 0 {
				let (name, _) = self.0.split_at(ii);
				let (_, next) = self.0.split_at(ii + 1);
				self.0 = next;
				return Some(XattrName::new_unchecked(name));
			}
		}
		let name = XattrName::new_unchecked(self.0);
		self.0 = &[];
		Some(name)
	}
}

struct DebugListxattrNames<'a>(&'a ListxattrBuf<'a>);

impl fmt::Debug for DebugListxattrNames<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_list()
			.entries(XattrNamesIter::new(self.0))
			.finish()
	}
}

impl encode::EncodeReply for ListxattrResponse<'_> {
	fn encode<S: encode::SendOnce>(
		&self,
		send: S,
		request_id: u64,
		_version_minor: u32,
	) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, request_id);
		match &self.buf {
			ListxattrBuf::SizeOnly { size } => {
				enc.encode_sized(&fuse_kernel::fuse_getxattr_out {
					size: *size,
					padding: 0,
				})
			},
			#[cfg(feature = "std")]
			ListxattrBuf::Owned { cap, .. } => enc.encode_bytes(&cap),
			ListxattrBuf::Borrowed { cap, size } => {
				let (bytes, _) = cap.split_at(*size);
				enc.encode_bytes(bytes)
			},
		}
	}
}

// }}}
