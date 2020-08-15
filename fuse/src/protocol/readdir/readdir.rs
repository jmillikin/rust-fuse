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

use core::num;

use crate::protocol::prelude::*;
use crate::protocol::read::fuse_read_in_v7p1;

#[cfg(test)]
mod readdir_test;

// ReaddirRequest {{{

/// Request type for [`FuseHandlers::readdir`].
///
/// [`FuseHandlers::readdir`]: ../trait.FuseHandlers.html#method.readdir
pub struct ReaddirRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	size: u32,
	cursor: Option<num::NonZeroU64>,
	handle: u64,
	flags: u32,
}

impl ReaddirRequest<'_> {
	pub fn node_id(&self) -> NodeId {
		self.node_id
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	pub fn cursor(&self) -> Option<num::NonZeroU64> {
		self.cursor
	}

	/// The value passed to [`OpendirResponse::set_handle`], or zero if not set.
	///
	/// [`OpendirResponse::set_handle`]: struct.OpendirResponse.html#method.set_handle
	pub fn handle(&self) -> u64 {
		self.handle
	}

	/// Platform-specific flags passed to [`FuseHandlers::opendir`]. See
	/// [`OpendirRequest::flags`] for details.
	///
	/// [`FuseHandlers::opendir`]: ../trait.FuseHandlers.html#method.opendir
	/// [`OpendirRequest::flags`]: struct.OpendirRequest.html#method.flags
	pub fn flags(&self) -> u32 {
		self.flags
	}
}

impl fmt::Debug for ReaddirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirRequest")
			.field("node_id", &self.node_id)
			.field("size", &self.size)
			.field("cursor", &format_args!("{:?}", self.cursor))
			.field("handle", &self.handle)
			.field("flags", &DebugHexU32(self.flags))
			.finish()
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ReaddirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_READDIR);

		let node_id = try_node_id(header.nodeid)?;

		// FUSE v7.9 added new fields to `fuse_read_in`.
		if dec.version().minor() < 9 {
			let raw: &'a fuse_read_in_v7p1 = dec.next_sized()?;
			return Ok(Self {
				phantom: PhantomData,
				node_id,
				size: raw.size,
				cursor: num::NonZeroU64::new(raw.offset),
				handle: raw.fh,
				flags: 0,
			});
		}

		let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id,
			size: raw.size,
			cursor: num::NonZeroU64::new(raw.offset),
			handle: raw.fh,
			flags: raw.flags,
		})
	}
}

// }}}

// ReaddirResponse {{{

/// Response type for [`FuseHandlers::readdir`].
///
/// [`FuseHandlers::readdir`]: ../trait.FuseHandlers.html#method.readdir
pub struct ReaddirResponse<'a> {
	buf: Option<ReaddirBuf<'a>>,
}

impl ReaddirResponse<'_> {
	/// An empty `ReaddirResponse` that cannot have entries added to it.
	///
	/// This is useful for returning end-of-stream responses.
	pub const EMPTY: &'static ReaddirResponse<'static> =
		&ReaddirResponse { buf: None };
}

impl<'a> ReaddirResponse<'a> {
	/// Constructs a new, empty `ReaddirResponse` that will grow up to the given
	/// maximum size.
	///
	/// # Examples
	///
	/// ```
	/// fn readdir(
	/// 	request: &fuse::ReaddirRequest,
	/// 	respond: impl for<'a> fuse::RespondOnce<fuse::ReaddirResponse<'a>>,
	/// ) {
	/// 	let mut response = fuse::ReaddirResponse::with_max_size(request.size());
	/// 	/* fill in response */
	/// 	respond.ok(&response);
	/// }
	/// ```
	pub fn with_max_size(max_size: u32) -> ReaddirResponse<'a> {
		let max_size = max_size as usize;
		Self {
			buf: Some(ReaddirBuf::Owned {
				cap: Vec::new(),
				max_size,
			}),
		}
	}

	/// Constructs a new, empty `ReaddirResponse` that will use the given buffer
	/// as capacity. The caller is responsible for allocating a buffer of the
	/// appropriate size and alignment.
	///
	/// # Panics
	///
	/// Panics if `buf` is not sufficiently aligned. The minimum alignment is
	/// `align_of::<u64>()`.
	///
	/// # Examples
	///
	/// ```
	/// # fn new_aligned_buf(_size: u32) -> Vec<u8> { Vec::new() }
	/// fn readdir(
	/// 	request: &fuse::ReaddirRequest,
	/// 	respond: impl for<'a> fuse::RespondOnce<fuse::ReaddirResponse<'a>>,
	/// ) {
	/// 	let mut buf = new_aligned_buf(request.size());
	/// 	let mut response = fuse::ReaddirResponse::with_capacity(&mut buf);
	/// 	/* fill in response */
	/// 	respond.ok(&response);
	/// }
	/// ```
	pub fn with_capacity(capacity: &'a mut [u8]) -> ReaddirResponse<'a> {
		let offset = capacity.as_ptr().align_offset(mem::align_of::<u64>());
		if offset != 0 {
			panic!(
				"ReaddirResponse::with_capacity() requires an 8-byte aligned buffer."
			);
		}
		Self {
			buf: Some(ReaddirBuf::Borrowed {
				cap: capacity,
				size: 0,
			}),
		}
	}

	pub fn add_entry(
		&mut self,
		node_id: NodeId,
		name: &NodeName,
		cursor: num::NonZeroU64,
	) -> ReaddirEntry {
		self.try_add_entry(node_id, name, cursor).unwrap()
	}

	pub fn try_add_entry(
		&mut self,
		node_id: NodeId,
		name: &NodeName,
		cursor: num::NonZeroU64,
	) -> Option<ReaddirEntry> {
		let name = name.as_bytes();
		let response_buf = self.buf.as_mut()?;
		let dirent_buf = response_buf.try_alloc_dirent(name)?;

		// From here on `try_add_entry()` must not fail, or the response buffer
		// would contain uninitialized bytes.

		unsafe {
			let dirent_ptr = dirent_buf as *mut fuse_kernel::fuse_dirent;
			let name_ptr =
				dirent_buf.add(size_of::<fuse_kernel::fuse_dirent>());
			let padding_ptr = name_ptr.add(name.len());

			dirent_ptr.write(fuse_kernel::fuse_dirent {
				ino: node_id.get(),
				off: cursor.get(),
				namelen: name.len() as u32,
				r#type: 0,
				name: [],
			});

			ptr::copy_nonoverlapping(name.as_ptr(), name_ptr, name.len());
			let padding_len = (8 - (name.len() % 8)) % 8;
			if padding_len > 0 {
				ptr::write_bytes(padding_ptr, 0, padding_len);
			}

			Some(ReaddirEntry {
				dirent: &mut *dirent_ptr,
			})
		}
	}
}

enum ReaddirBuf<'a> {
	Owned { cap: Vec<u8>, max_size: usize },
	Borrowed { cap: &'a mut [u8], size: usize },
}

impl ReaddirBuf<'_> {
	fn try_alloc_dirent(&mut self, name: &[u8]) -> Option<*mut u8> {
		debug_assert!(name.len() > 0);

		let padding_len = (8 - (name.len() % 8)) % 8;
		let overhead = padding_len + size_of::<fuse_kernel::fuse_dirent>();
		let entry_size = overhead + name.len();

		let entry_buf = match self {
			ReaddirBuf::Borrowed {
				cap,
				size: size_ref,
			} => {
				let current_size: usize = *size_ref;
				let new_size = current_size.checked_add(entry_size)?;
				if new_size > cap.len() {
					return None;
				}
				let (_, remaining_cap) = cap.split_at_mut(current_size);
				let (entry_buf, _) = remaining_cap.split_at_mut(entry_size);
				*size_ref = new_size;
				entry_buf
			},

			Self::Owned { cap, max_size } => {
				let current_size = cap.len();
				let new_size = current_size.checked_add(entry_size)?;
				if new_size > *max_size {
					return None;
				}
				cap.resize(new_size, 0u8);
				let (_, entry_buf) = cap.split_at_mut(current_size);
				entry_buf
			},
		};

		debug_assert!(
			entry_buf.len() == entry_size,
			"{} == {}",
			entry_buf.len(),
			entry_size,
		);
		Some(entry_buf.as_mut_ptr())
	}

	fn foreach_dirent<F>(&self, mut f: F)
	where
		F: FnMut(&fuse_kernel::fuse_dirent),
	{
		let mut buf = match &self {
			Self::Owned { cap, .. } => cap.as_slice(),
			Self::Borrowed { cap, size } => {
				let (used, _) = cap.split_at(*size);
				used
			},
		};

		const ENTRY_SIZE: usize = mem::size_of::<fuse_kernel::fuse_dirent>();
		while buf.len() > 0 {
			debug_assert!(buf.len() >= ENTRY_SIZE);
			let dirent =
				unsafe { &*(buf.as_ptr() as *const fuse_kernel::fuse_dirent) };
			let padding = ((8 - (dirent.namelen % 8)) % 8) as usize;
			let entry_span = ENTRY_SIZE + (dirent.namelen as usize) + padding;
			let (_, next) = buf.split_at(entry_span);
			f(dirent);
			buf = next;
		}
	}
}

pub struct ReaddirEntry<'a> {
	dirent: &'a mut fuse_kernel::fuse_dirent,
}

impl ReaddirEntry<'_> {
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.dirent.ino) }
	}

	pub fn name(&self) -> &[u8] {
		dirent_name(self.dirent)
	}

	pub fn cursor(&self) -> num::NonZeroU64 {
		unsafe { num::NonZeroU64::new_unchecked(self.dirent.off) }
	}

	pub fn file_type(&self) -> FileType {
		FileType(self.dirent.r#type)
	}

	pub fn set_file_type(&mut self, file_type: FileType) -> &mut Self {
		self.dirent.r#type = file_type.0;
		self
	}
}

fn dirent_name(dirent: &fuse_kernel::fuse_dirent) -> &[u8] {
	unsafe {
		core::slice::from_raw_parts(
			&dirent.name as *const u8,
			dirent.namelen as usize,
		)
	}
}

fn dirent_fmt(
	dirent: &fuse_kernel::fuse_dirent,
	fmt: &mut fmt::Formatter,
) -> fmt::Result {
	fmt.debug_struct("ReaddirEntry")
		.field("node_id", &dirent.ino)
		.field("cursor", &dirent.off)
		.field("file_type", &FileType(dirent.r#type))
		.field("name", &DebugBytesAsString(dirent_name(dirent)))
		.finish()
}

impl fmt::Debug for ReaddirEntry<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		dirent_fmt(self.dirent, fmt)
	}
}

impl fmt::Debug for ReaddirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirResponse")
			.field(
				"entries",
				&DebugClosure(|fmt| {
					let mut list = fmt.debug_list();
					match self.buf {
						None => {},
						Some(ref buf) => buf.foreach_dirent(|dirent| {
							list.entry(&DebugClosure(|fmt| {
								dirent_fmt(dirent, fmt)
							}));
						}),
					};
					list.finish()
				}),
			)
			.finish()
	}
}

impl fuse_io::EncodeResponse for ReaddirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Error> {
		let buf = match &self.buf {
			None => return enc.encode_header_only(),
			Some(x) => x,
		};
		match buf {
			ReaddirBuf::Owned { cap, .. } => enc.encode_bytes(&cap),
			ReaddirBuf::Borrowed { cap, size } => {
				let (bytes, _) = cap.split_at(*size);
				enc.encode_bytes(bytes)
			},
		}
	}
}

// }}}
