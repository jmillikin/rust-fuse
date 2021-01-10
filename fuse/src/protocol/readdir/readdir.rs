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

// ReaddirRequest {{{

/// Request type for [`FuseHandlers::readdir`].
///
/// [`FuseHandlers::readdir`]: ../../trait.FuseHandlers.html#method.readdir
pub struct ReaddirRequest<'a> {
	phantom: PhantomData<&'a ()>,
	node_id: NodeId,
	size: u32,
	cursor: Option<num::NonZeroU64>,
	handle: u64,
	opendir_flags: u32,
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
	/// [`FuseHandlers::opendir`]: ../../trait.FuseHandlers.html#method.opendir
	/// [`OpendirRequest::flags`]: struct.OpendirRequest.html#method.flags
	pub fn opendir_flags(&self) -> u32 {
		self.opendir_flags
	}
}

impl fmt::Debug for ReaddirRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("ReaddirRequest")
			.field("node_id", &self.node_id)
			.field("size", &self.size)
			.field("cursor", &format_args!("{:?}", self.cursor))
			.field("handle", &self.handle)
			.field("opendir_flags", &DebugHexU32(self.opendir_flags))
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
				opendir_flags: 0,
			});
		}

		let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;
		Ok(Self {
			phantom: PhantomData,
			node_id,
			size: raw.size,
			cursor: num::NonZeroU64::new(raw.offset),
			handle: raw.fh,
			opendir_flags: raw.flags,
		})
	}
}

// }}}

// ReaddirResponse {{{

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ReaddirError {
	kind: ReaddirErrorKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ReaddirErrorKind {
	ExceedsCapacity(usize, usize),
	OverflowsUsize,
}

impl ReaddirError {
	fn exceeds_capacity(response_size: usize, capacity: usize) -> ReaddirError {
		ReaddirError {
			kind: ReaddirErrorKind::ExceedsCapacity(response_size, capacity),
		}
	}

	fn overflows_usize() -> ReaddirError {
		ReaddirError {
			kind: ReaddirErrorKind::OverflowsUsize,
		}
	}
}

impl fmt::Display for ReaddirError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		fmt::Debug::fmt(self, fmt)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for ReaddirError {}

/// Response type for [`FuseHandlers::readdir`].
///
/// [`FuseHandlers::readdir`]: ../../trait.FuseHandlers.html#method.readdir
pub struct ReaddirResponse<'a> {
	buf: ReaddirBuf<'a, fuse_kernel::fuse_dirent>,
}

impl ReaddirResponse<'_> {
	/// An empty `ReaddirResponse` that cannot have entries added to it.
	///
	/// This is useful for returning end-of-stream responses.
	pub const EMPTY: &'static ReaddirResponse<'static> = &ReaddirResponse {
		buf: ReaddirBuf::None,
	};
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
	/// 	respond: impl for<'a> fuse::Respond<fuse::ReaddirResponse<'a>>,
	/// ) {
	/// 	let mut response = fuse::ReaddirResponse::with_max_size(request.size());
	/// 	/* fill in response */
	/// 	respond.ok(&response);
	/// }
	/// ```
	#[cfg(feature = "std")]
	#[cfg_attr(doc, doc(cfg(feature = "std")))]
	pub fn with_max_size(max_size: u32) -> ReaddirResponse<'a> {
		let max_size = max_size as usize;
		Self {
			buf: ReaddirBuf::Owned {
				cap: Vec::new(),
				max_size,
			},
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
	/// 	respond: impl for<'a> fuse::Respond<fuse::ReaddirResponse<'a>>,
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
			buf: ReaddirBuf::Borrowed {
				cap: capacity,
				size: 0,
				phantom: PhantomData,
			},
		}
	}

	pub fn entries(&self) -> impl Iterator<Item = &ReaddirEntry> {
		ReaddirEntriesIter::new(&self.buf)
	}

	pub fn add_entry(
		&mut self,
		node_id: NodeId,
		name: &NodeName,
		cursor: num::NonZeroU64,
	) -> &mut ReaddirEntry {
		self.try_add_entry(node_id, name, cursor).unwrap()
	}

	pub fn try_add_entry(
		&mut self,
		node_id: NodeId,
		name: &NodeName,
		cursor: num::NonZeroU64,
	) -> Result<&mut ReaddirEntry, ReaddirError> {
		let name = name.as_bytes();
		let dirent_buf = self.buf.try_alloc_dirent(name)?;

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
				r#type: FileType::Unknown.as_bits(),
				name: [],
			});

			ptr::copy_nonoverlapping(name.as_ptr(), name_ptr, name.len());
			let padding_len = (8 - (name.len() % 8)) % 8;
			if padding_len > 0 {
				ptr::write_bytes(padding_ptr, 0, padding_len);
			}

			Ok(ReaddirEntry::new_ref_mut(&mut *dirent_ptr))
		}
	}
}

enum ReaddirBuf<'a, Dirent> {
	None,
	#[cfg(feature = "std")]
	Owned {
		cap: Vec<u8>,
		max_size: usize,
	},
	Borrowed {
		cap: &'a mut [u8],
		size: usize,
		phantom: PhantomData<&'a Dirent>,
	},
}

trait DirentT {
	fn namelen(&self) -> u32;
}

impl DirentT for fuse_kernel::fuse_dirent {
	fn namelen(&self) -> u32 {
		self.namelen
	}
}

impl<Dirent: DirentT> ReaddirBuf<'_, Dirent> {
	fn try_alloc_dirent(
		&mut self,
		name: &[u8],
	) -> Result<*mut u8, ReaddirError> {
		debug_assert!(name.len() > 0);

		let padding_len = (8 - (name.len() % 8)) % 8;
		let overhead = padding_len + size_of::<Dirent>();
		let entry_size = overhead + name.len();

		let entry_buf = match self {
			ReaddirBuf::None => {
				return Err(ReaddirError::exceeds_capacity(entry_size, 0));
			},
			ReaddirBuf::Borrowed {
				cap,
				size: size_ref,
				..
			} => {
				let current_size: usize = *size_ref;
				let new_size = match current_size.checked_add(entry_size) {
					Some(x) => x,
					None => return Err(ReaddirError::overflows_usize()),
				};
				if new_size > cap.len() {
					return Err(ReaddirError::exceeds_capacity(
						new_size,
						cap.len(),
					));
				}
				let (_, remaining_cap) = cap.split_at_mut(current_size);
				let (entry_buf, _) = remaining_cap.split_at_mut(entry_size);
				*size_ref = new_size;
				entry_buf
			},

			#[cfg(feature = "std")]
			Self::Owned { cap, max_size } => {
				let current_size = cap.len();
				let new_size = match current_size.checked_add(entry_size) {
					Some(x) => x,
					None => return Err(ReaddirError::overflows_usize()),
				};
				if new_size > *max_size {
					return Err(ReaddirError::exceeds_capacity(
						new_size, *max_size,
					));
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
		Ok(entry_buf.as_mut_ptr())
	}

	fn next_dirent(&self, offset: usize) -> Option<(&Dirent, usize)> {
		let mut buf = match &self {
			Self::None => &[],
			#[cfg(feature = "std")]
			Self::Owned { cap, .. } => cap.as_slice(),
			Self::Borrowed { cap, size, .. } => {
				let (used, _) = cap.split_at(*size);
				used
			},
		};
		if offset == buf.len() {
			return None;
		}
		if offset > 0 {
			let (_, buf_offset) = buf.split_at(offset);
			buf = buf_offset;
		}

		let dirent_size = mem::size_of::<Dirent>();
		debug_assert!(buf.len() >= dirent_size);
		let dirent = unsafe { &*(buf.as_ptr() as *const Dirent) };
		let name_len = dirent.namelen() as usize;
		let padding = (8 - (name_len % 8)) % 8;
		let entry_span = dirent_size + name_len + padding;

		return Some((dirent, offset + entry_span));
	}
}

#[repr(transparent)]
pub struct ReaddirEntry(fuse_kernel::fuse_dirent);

impl ReaddirEntry {
	pub(crate) fn new_ref(raw: &fuse_kernel::fuse_dirent) -> &ReaddirEntry {
		unsafe {
			&*(raw as *const fuse_kernel::fuse_dirent as *const ReaddirEntry)
		}
	}

	pub(crate) fn new_ref_mut(
		raw: &mut fuse_kernel::fuse_dirent,
	) -> &mut ReaddirEntry {
		unsafe {
			&mut *(raw as *mut fuse_kernel::fuse_dirent as *mut ReaddirEntry)
		}
	}

	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.0.ino) }
	}

	pub fn name(&self) -> &[u8] {
		dirent_name(&self.0)
	}

	pub fn cursor(&self) -> num::NonZeroU64 {
		unsafe { num::NonZeroU64::new_unchecked(self.0.off) }
	}

	pub fn file_type(&self) -> FileType {
		dirent_type(&self.0)
	}

	pub fn set_file_type(&mut self, file_type: FileType) {
		self.0.r#type = file_type.as_bits();
	}
}

fn dirent_type(dirent: &fuse_kernel::fuse_dirent) -> FileType {
	match FileType::from_bits(dirent.r#type) {
		Some(t) => t,
		None => unsafe {
			core::hint::unreachable_unchecked()
		},
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
		.field("file_type", &dirent_type(dirent))
		.field("name", &DebugBytesAsString(dirent_name(dirent)))
		.finish()
}

impl fmt::Debug for ReaddirEntry {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		dirent_fmt(&self.0, fmt)
	}
}

struct ReaddirEntriesIter<'a> {
	buf: &'a ReaddirBuf<'a, fuse_kernel::fuse_dirent>,
	offset: usize,
}

impl<'a> ReaddirEntriesIter<'a> {
	fn new(buf: &'a ReaddirBuf<'a, fuse_kernel::fuse_dirent>) -> Self {
		Self { buf, offset: 0 }
	}
}

impl<'a> core::iter::Iterator for ReaddirEntriesIter<'a> {
	type Item = &'a ReaddirEntry;

	fn next(&mut self) -> Option<&'a ReaddirEntry> {
		match self.buf.next_dirent(self.offset) {
			None => None,
			Some((dirent, new_offset)) => {
				self.offset = new_offset;
				Some(ReaddirEntry::new_ref(dirent))
			},
		}
	}
}

impl fmt::Debug for ReaddirResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let entries = DebugClosure(|fmt| {
			fmt.debug_list()
				.entries(ReaddirEntriesIter::new(&self.buf))
				.finish()
		});
		fmt.debug_struct("ReaddirResponse")
			.field("entries", &entries)
			.finish()
	}
}

impl fuse_io::EncodeResponse for ReaddirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> Result<(), Chan::Error> {
		match &self.buf {
			ReaddirBuf::None => enc.encode_header_only(),
			#[cfg(feature = "std")]
			ReaddirBuf::Owned { cap, .. } => enc.encode_bytes(&cap),
			ReaddirBuf::Borrowed { cap, size, .. } => {
				let (bytes, _) = cap.split_at(*size);
				enc.encode_bytes(bytes)
			},
		}
	}
}

// }}}
