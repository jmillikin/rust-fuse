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

use crate::protocol::node;
use crate::protocol::prelude::*;
use crate::protocol::read::fuse_read_in_v7p1;

#[cfg(test)]
mod readdir_test;

// ReaddirRequest {{{

/// **\[UNSTABLE\]**
pub struct ReaddirRequest<'a> {
	header: &'a fuse_kernel::fuse_in_header,
	raw: fuse_kernel::fuse_read_in,
}

impl ReaddirRequest<'_> {
	pub fn node_id(&self) -> u64 {
		self.header.nodeid
	}

	pub fn handle(&self) -> u64 {
		self.raw.fh
	}

	pub fn offset(&self) -> u64 {
		self.raw.offset
	}

	pub fn size(&self) -> u32 {
		self.raw.size
	}

	pub fn lock_owner(&self) -> Option<u64> {
		if self.raw.read_flags & fuse_kernel::FUSE_READ_LOCKOWNER == 0 {
			return None;
		}
		Some(self.raw.lock_owner)
	}

	pub fn flags(&self) -> u32 {
		self.raw.flags
	}
}

impl<'a> fuse_io::DecodeRequest<'a> for ReaddirRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> io::Result<Self> {
		let header = dec.header();
		if header.opcode == fuse_kernel::FUSE_READDIRPLUS {
			let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;
			return Ok(Self { header, raw: *raw });
		}

		debug_assert!(header.opcode == fuse_kernel::FUSE_READDIR);

		// FUSE v7.9 added new fields to `fuse_read_in`.
		if dec.version().minor() < 9 {
			let raw: &'a fuse_read_in_v7p1 = dec.next_sized()?;
			return Ok(Self {
				header,
				raw: fuse_kernel::fuse_read_in {
					fh: raw.fh,
					offset: raw.offset,
					size: raw.size,
					read_flags: 0,
					lock_owner: 0,
					flags: 0,
					padding: 0,
				},
			});
		}

		let raw: &'a fuse_kernel::fuse_read_in = dec.next_sized()?;
		Ok(Self { header, raw: *raw })
	}
}

// }}}

// ReaddirResponse {{{

#[allow(dead_code)]
const FUSE_NAME_MAX_CFG_LINUX: usize = 1024;

#[allow(dead_code)]
const FUSE_NAME_MAX_CFG_FREEBSD: usize = 255;

#[cfg(target_os = "linux")]
pub const FUSE_NAME_MAX: usize = FUSE_NAME_MAX_CFG_LINUX;

#[cfg(target_os = "freebsd")]
pub const FUSE_NAME_MAX: usize = FUSE_NAME_MAX_CFG_FREEBSD;

const MAX_RESPONSE_SIZE: usize = 4096;

/// **\[UNSTABLE\]**
pub struct ReaddirResponse<'a> {
	phantom: PhantomData<&'a ()>,
	opcode: fuse_kernel::Opcode,
	response_size: usize,
	max_response_size: usize,

	// buf is semantically [u8; MAX_RESPONSE_SIZE], but defined as [u64] so
	// element pointers will be properly aligned for `fuse_dirent`.
	buf: [u64; MAX_RESPONSE_SIZE / 8],
}

impl<'a> ReaddirResponse<'a> {
	// TODO: fix the API here
	pub fn new(request: &ReaddirRequest) -> Self {
		ReaddirResponse {
			phantom: PhantomData,
			opcode: request.header.opcode,
			response_size: 0,
			max_response_size: cmp::min(
				request.raw.size,
				MAX_RESPONSE_SIZE as u32,
			) as usize,
			buf: unsafe { mem::uninitialized() },
		}
	}

	pub fn push(
		&mut self,
		node_id: node::NodeId,
		offset: u64,
		name: &CStr,
	) -> Option<Dirent> {
		let name = name.to_bytes();
		let name_len = name.len();
		assert!(name_len > 0);

		let op_readdirplus = self.opcode == fuse_kernel::FUSE_READDIRPLUS;
		let padding_len = (8 - (name_len % 8)) % 8;
		let overhead = padding_len
			+ (if op_readdirplus {
				size_of::<fuse_kernel::fuse_direntplus>()
			} else {
				size_of::<fuse_kernel::fuse_dirent>()
			});

		let buf_p = match self.buf_alloc(overhead.saturating_add(name_len)) {
			Some(p) => p,
			None => return None,
		};

		// From here on `push()` must not fail, or the response buffer would
		// contain uninitialized bytes.

		let entry = unsafe {
			let mut dirent_p: *mut u8 = buf_p;
			let entry_out_p: *mut u8 = buf_p;
			if op_readdirplus {
				dirent_p =
					dirent_p.add(size_of::<fuse_kernel::fuse_entry_out>());
				ptr::write_bytes(
					entry_out_p,
					0,
					size_of::<fuse_kernel::fuse_entry_out>(),
				);
			}
			ptr::write_bytes(
				dirent_p,
				0,
				size_of::<fuse_kernel::fuse_dirent>(),
			);
			let name_p: *mut u8 =
				dirent_p.add(size_of::<fuse_kernel::fuse_dirent>());
			let padding_p: *mut u8 = name_p.add(name_len);
			let name_ref = std::slice::from_raw_parts_mut(name_p, name_len);
			name_ref.copy_from_slice(name);
			if padding_len > 0 {
				ptr::write_bytes(padding_p, 0, padding_len);
			}

			Dirent {
				node_id,
				node: if dirent_p == entry_out_p {
					None
				} else {
					Some(&mut *(entry_out_p as *mut node::Node))
				},
				dirent: &mut *(dirent_p as *mut fuse_kernel::fuse_dirent),
				name: name_ref,
			}
		};
		entry.dirent.ino = node_id.get();
		entry.dirent.off = offset;
		entry.dirent.namelen = name_len as u32;
		entry.dirent.r#type = node::NodeKind::UNKNOWN.raw();
		Some(entry)
	}

	fn buf_alloc(&mut self, size: usize) -> Option<*mut u8> {
		debug_assert_eq!(size % 8, 0);

		let new_response_size: usize;
		match self.response_size.checked_add(size) {
			None => return None,
			Some(x) => new_response_size = x,
		};
		if new_response_size > self.max_response_size {
			return None;
		}
		let buf_p = self.buf.as_ptr() as *mut u8;
		let out = unsafe { buf_p.add(self.response_size) };
		self.response_size = new_response_size;
		Some(out)
	}
}

impl<'a> fmt::Debug for ReaddirResponse<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let dirents: Vec<Dirent<'a>> = Vec::new();
		// TODO
		fmt.debug_struct("ReaddirResponse")
			.field("dirents", &dirents)
			.finish()
	}
}

impl fuse_io::EncodeResponse for ReaddirResponse<'_> {
	fn encode_response<'a, Chan: fuse_io::Channel>(
		&'a self,
		enc: fuse_io::ResponseEncoder<Chan>,
	) -> std::io::Result<()> {
		enc.encode_bytes(unsafe {
			std::slice::from_raw_parts(
				self.buf.as_ptr() as *const u8,
				self.response_size,
			)
		})
	}
}

// }}}

// Dirent {{{

/// **\[UNSTABLE\]**
pub struct Dirent<'a> {
	node_id: node::NodeId,
	node: Option<&'a mut node::Node>,
	dirent: &'a mut fuse_kernel::fuse_dirent,
	name: &'a [u8],
}

impl Dirent<'_> {
	pub fn node_id(&self) -> node::NodeId {
		match self.node {
			None => self.node_id,
			Some(ref node) => match node.id() {
				None => self.node_id,
				Some(id) => id,
			},
		}
	}

	pub fn set_node_id(&mut self, node_id: node::NodeId) {
		self.node_id = node_id;
		self.dirent.ino = node_id.get();
		match self.node {
			None => {},
			Some(ref mut node) => match node.id() {
				None => {},
				Some(_) => node.set_id(node_id),
			},
		}
	}

	pub fn offset(&self) -> u64 {
		self.dirent.off
	}

	pub fn set_offset(&mut self, offset: u64) {
		self.dirent.off = offset;
	}

	pub fn node_kind(&self) -> node::NodeKind {
		let dirent_type = node::NodeKind::new(self.dirent.r#type);
		match self.node {
			None => dirent_type,
			Some(ref node) => match node.id() {
				None => dirent_type,
				Some(_) => node.kind(),
			},
		}
	}

	pub fn set_node_kind(&mut self, node_kind: node::NodeKind) {
		self.dirent.r#type = node_kind.raw();
		match self.node {
			None => {},
			Some(ref mut node) => match node.id() {
				None => {},
				Some(_) => node.set_kind(node_kind),
			},
		}
	}

	pub fn node_mut(&mut self) -> Option<&mut node::Node> {
		match &mut self.node {
			None => None,
			Some(node) => {
				if node.id() == None {
					node.set_id(self.node_id);
					node.set_kind(node::NodeKind::new(self.dirent.r#type));
				}
				Some(node)
			},
		}
	}
}

impl Drop for Dirent<'_> {
	fn drop(&mut self) {
		match self.node {
			None => {},
			Some(ref node) => match node.id() {
				None => {},
				Some(node_id) => {
					self.dirent.ino = node_id.get();
					self.dirent.r#type = node.kind().raw();
				},
			},
		}
	}
}

impl fmt::Debug for Dirent<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Dirent")
			.field("node_id", &self.node_id())
			.field("offset", &self.dirent.off)
			.field("node_kind", &self.node_kind())
			.field("node", &self.node)
			.field("name", &self.name) // TODO: cstr style
			.finish()
	}
}

// }}}
