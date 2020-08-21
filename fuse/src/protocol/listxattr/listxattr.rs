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

#[cfg(test)]
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

impl<'a> fuse_io::DecodeRequest<'a> for ListxattrRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_LISTXATTR);

		let raw: &fuse_kernel::fuse_getxattr_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id: try_node_id(header.nodeid)?,
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
	request_size: Option<num::NonZeroU32>,
	raw: fuse_kernel::fuse_getxattr_out,
	buf: ListxattrBuf<'a>,
}

impl<'a> ListxattrResponse<'a> {
	#[cfg(feature = "std")]
	#[cfg_attr(doc, doc(cfg(feature = "std")))]
	pub fn new(request_size: Option<num::NonZeroU32>) -> ListxattrResponse<'a> {
		Self {
			request_size,
			raw: Default::default(),
			buf: ListxattrBuf::Owned { cap: Vec::new() },
		}
	}

	pub fn with_capacity(
		request_size: Option<num::NonZeroU32>,
		capacity: &'a mut [u8],
	) -> ListxattrResponse<'a> {
		Self {
			request_size,
			raw: Default::default(),
			buf: ListxattrBuf::Borrowed {
				cap: capacity,
				size: 0,
			},
		}
	}

	pub fn request_size(&self) -> Option<num::NonZeroU32> {
		self.request_size
	}

	pub fn add_name(&mut self, name: &XattrName) {
		self.try_add_name(name).unwrap()
	}

	pub fn try_add_name(&mut self, name: &XattrName) -> Option<()> {
		use crate::XATTR_LIST_MAX;

		let name = name.as_bytes();
		let name_len = name.len() as usize;
		let name_buf_len = name_len + 1; // includes NUL terminator

		let mut request_size = match self.request_size {
			None => {
				// Don't actually copy any bytes around, just keep track of the
				// response size.
				if self.raw.size as usize + name_buf_len > XATTR_LIST_MAX {
					return None;
				}
				self.raw.size += name_buf_len as u32;
				return Some(());
			},
			Some(x) => x.get() as usize,
		};

		if request_size > XATTR_LIST_MAX {
			request_size = XATTR_LIST_MAX;
		}

		let name_buf = match &mut self.buf {
			ListxattrBuf::Borrowed {
				cap,
				size: size_ref,
			} => {
				let current_size = *size_ref;
				let new_size = current_size + name_buf_len;
				if new_size > cap.len() || new_size > request_size {
					return None;
				}
				let (_, remaining_cap) = cap.split_at_mut(current_size);
				let (name_buf, _) = remaining_cap.split_at_mut(name_buf_len);
				*size_ref = new_size;
				name_buf
			},
			#[cfg(feature = "std")]
			ListxattrBuf::Owned { cap } => {
				let current_size = cap.len();
				let new_size = current_size + name_buf_len;
				if new_size > request_size {
					return None;
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
		Some(())
	}
}

enum ListxattrBuf<'a> {
	#[cfg(feature = "std")]
	Owned {
		cap: Vec<u8>,
	},
	Borrowed {
		cap: &'a mut [u8],
		size: usize,
	},
}

impl fmt::Debug for ListxattrResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut out = fmt.debug_struct("ListxattrResponse");
		out.field("request_size", &format_args!("{:?}", &self.request_size));
		match self.request_size {
			None => {
				out.field("size", &self.raw.size);
				let names: &[u8] = &[];
				out.field("names", &names);
			},
			Some(_) => match &self.buf {
				#[cfg(feature = "std")]
				ListxattrBuf::Owned { cap } => {
					out.field("size", &cap.len());
					out.field("names", &DebugListxattrNames(&cap));
				},
				ListxattrBuf::Borrowed { cap, size } => {
					let (bytes, _) = cap.split_at(*size);
					out.field("size", size);
					out.field("names", &DebugListxattrNames(&bytes));
				},
			},
		}

		out.finish()
	}
}

struct DebugListxattrNames<'a>(&'a [u8]);

impl fmt::Debug for DebugListxattrNames<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut out = fmt.debug_list();
		for chunk in self.0.split(|&b| b == 0) {
			if chunk.len() > 0 {
				out.entry(&DebugBytesAsString(chunk));
			}
		}
		out.finish()
	}
}

impl fuse_io::EncodeResponse for ListxattrResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		if self.request_size.is_none() {
			return enc.encode_sized(&self.raw);
		}

		match &self.buf {
			#[cfg(feature = "std")]
			ListxattrBuf::Owned { cap } => enc.encode_bytes(&cap),
			ListxattrBuf::Borrowed { cap, size } => {
				let (bytes, _) = cap.split_at(*size);
				enc.encode_bytes(bytes)
			},
		}
	}
}

// }}}
