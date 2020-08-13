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

use core::{cmp, fmt, num, time};
use std::ffi;

use crate::internal::fuse_io;
use crate::internal::fuse_kernel;

// NodeId {{{

/// **\[UNSTABLE\]**
#[repr(C)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub struct NodeId(num::NonZeroU64);

impl NodeId {
	pub const ROOT: NodeId = NodeId(unsafe {
		num::NonZeroU64::new_unchecked(fuse_kernel::FUSE_ROOT_ID)
	});

	pub fn new(id: u64) -> Option<NodeId> {
		num::NonZeroU64::new(id).map(|bits| NodeId(bits))
	}

	pub fn get(&self) -> u64 {
		self.0.get()
	}
}

impl fmt::Debug for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::Display for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::Binary for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::LowerHex for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::UpperHex for NodeId {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

// }}}

// NodeName {{{

#[derive(Hash)]
#[repr(transparent)]
pub struct NodeName {
	pub(crate) bytes: [u8],
}

impl NodeName {
	#[rustfmt::skip]
	pub const NAME_MAX: usize = {
		#[cfg(target_os = "linux")]   { 1024 }
		#[cfg(target_os = "freebsd")] {  255 }
	};

	pub(crate) fn new<'a>(bytes: fuse_io::NulTerminatedBytes<'a>) -> &'a Self {
		let bytes = bytes.to_bytes_without_nul();
		unsafe { &*(bytes as *const [u8] as *const NodeName) }
	}

	pub fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a Self> {
		let len = bytes.len();
		if len == 0 || len > Self::NAME_MAX {
			return None;
		}
		if bytes.contains(&b'/') {
			return None;
		}
		Some(unsafe { &*(bytes as *const [u8] as *const NodeName) })
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}
}

impl fmt::Debug for NodeName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for NodeName {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use core::fmt::Debug;
		super::prelude::DebugBytesAsString(&self.bytes).fmt(fmt)
	}
}

impl Eq for NodeName {}

impl PartialEq for NodeName {
	fn eq(&self, other: &NodeName) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for NodeName {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<NodeName> for [u8] {
	fn eq(&self, other: &NodeName) -> bool {
		self.eq(other.as_bytes())
	}
}

impl PartialOrd for NodeName {
	fn partial_cmp(&self, other: &NodeName) -> Option<cmp::Ordering> {
		self.as_bytes().partial_cmp(&other.as_bytes())
	}
}

impl Ord for NodeName {
	fn cmp(&self, other: &NodeName) -> cmp::Ordering {
		self.as_bytes().cmp(&other.as_bytes())
	}
}

// }}}

// NodeKind {{{

/// **\[UNSTABLE\]**
#[derive(Eq, PartialEq, Clone, Copy)]
pub struct NodeKind(u32);

const DT_UNKNOWN: u32 = 0;
const DT_FIFO: u32 = 1;
const DT_CHR: u32 = 2;
const DT_DIR: u32 = 4;
const DT_BLK: u32 = 6;
const DT_REG: u32 = 8;
const DT_LNK: u32 = 10;
const DT_SOCK: u32 = 12;
const DT_WHT: u32 = 14;

impl NodeKind {
	pub const UNKNOWN: NodeKind = NodeKind(DT_UNKNOWN);
	pub const FIFO: NodeKind = NodeKind(DT_FIFO);
	pub const CHR: NodeKind = NodeKind(DT_CHR);
	pub const DIR: NodeKind = NodeKind(DT_DIR);
	pub const BLK: NodeKind = NodeKind(DT_BLK);
	pub const REG: NodeKind = NodeKind(DT_REG);
	pub const LNK: NodeKind = NodeKind(DT_LNK);
	pub const SOCK: NodeKind = NodeKind(DT_SOCK);
	pub const WHT: NodeKind = NodeKind(DT_WHT);

	pub(crate) fn new(kind: u32) -> Self {
		Self(kind)
	}

	pub(crate) fn raw(&self) -> u32 {
		self.0
	}
}

impl fmt::Debug for NodeKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for NodeKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			NodeKind::UNKNOWN => fmt.write_str("UNKNOWN"),
			NodeKind::FIFO => fmt.write_str("FIFO"),
			NodeKind::CHR => fmt.write_str("CHR"),
			NodeKind::DIR => fmt.write_str("DIR"),
			NodeKind::BLK => fmt.write_str("BLK"),
			NodeKind::REG => fmt.write_str("REG"),
			NodeKind::LNK => fmt.write_str("LNK"),
			NodeKind::SOCK => fmt.write_str("SOCK"),
			NodeKind::WHT => fmt.write_str("WHT"),
			_ => write!(fmt, "{:#010X}", self.0),
		}
	}
}

impl fmt::Binary for NodeKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::LowerHex for NodeKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

impl fmt::UpperHex for NodeKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(fmt)
	}
}

// }}}

// NodeAttr {{{

/// **\[UNSTABLE\]**
pub struct NodeAttr(fuse_kernel::fuse_attr);

impl NodeAttr {
	pub fn size(&self) -> u64 {
		self.0.size
	}

	pub fn set_size(&mut self, size: u64) {
		self.0.size = size;
	}

	pub fn blocks(&self) -> u64 {
		self.0.blocks
	}

	pub fn set_blocks(&mut self, blocks: u64) {
		self.0.blocks = blocks;
	}

	pub fn atime(&self) -> time::Duration {
		time::Duration::new(self.0.atime, self.0.atimensec)
	}

	pub fn set_atime(&mut self, atime: time::Duration) {
		self.0.atime = atime.as_secs();
		self.0.atimensec = atime.subsec_nanos();
	}

	pub fn mtime(&self) -> time::Duration {
		time::Duration::new(self.0.mtime, self.0.mtimensec)
	}

	pub fn set_mtime(&mut self, mtime: time::Duration) {
		self.0.mtime = mtime.as_secs();
		self.0.mtimensec = mtime.subsec_nanos();
	}

	pub fn ctime(&self) -> time::Duration {
		time::Duration::new(self.0.ctime, self.0.ctimensec)
	}

	pub fn set_ctime(&mut self, ctime: time::Duration) {
		self.0.ctime = ctime.as_secs();
		self.0.ctimensec = ctime.subsec_nanos();
	}

	pub fn set_mode(&mut self, mode: u32) {
		self.0.mode = mode;
	}

	pub fn set_nlink(&mut self, nlink: u32) {
		self.0.nlink = nlink;
	}

	pub fn set_user_id(&mut self, user_id: u32) {
		self.0.uid = user_id;
	}

	pub fn set_group_id(&mut self, group_id: u32) {
		self.0.gid = group_id;
	}

	pub fn set_rdev(&mut self, rdev: u32) {
		self.0.rdev = rdev;
	}

	pub fn set_blksize(&mut self, blksize: u32) {
		self.0.blksize = blksize;
	}

	pub(crate) fn new_ref(raw: &fuse_kernel::fuse_attr) -> &Self {
		let p = raw as *const fuse_kernel::fuse_attr as *const Self;
		unsafe { &*p }
	}

	pub(crate) fn new_ref_mut(raw: &mut fuse_kernel::fuse_attr) -> &mut Self {
		let p = raw as *mut fuse_kernel::fuse_attr as *mut Self;
		unsafe { &mut *p }
	}
}

impl fmt::Debug for NodeAttr {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("NodeAttr")
			.field("size", &self.0.size)
			.field("blocks", &self.0.blocks)
			.field("atime", &self.atime())
			.field("mtime", &self.mtime())
			.field("ctime", &self.ctime())
			.field("mode", &self.0.mode)
			.field("nlink", &self.0.nlink)
			.field("uid", &self.0.uid)
			.field("gid", &self.0.gid)
			.field("rdev", &self.0.rdev)
			.field("blksize", &self.0.blksize)
			.finish()
	}
}

// }}}

// Node {{{

/// **\[UNSTABLE\]**
pub struct Node(fuse_kernel::fuse_entry_out);

impl Node {
	pub(crate) fn new_ref(raw: &fuse_kernel::fuse_entry_out) -> &Node {
		unsafe { &*(raw as *const fuse_kernel::fuse_entry_out as *const Node) }
	}

	pub(crate) fn new_ref_mut(
		raw: &mut fuse_kernel::fuse_entry_out,
	) -> &mut Node {
		unsafe { &mut *(raw as *mut fuse_kernel::fuse_entry_out as *mut Node) }
	}

	pub fn id(&self) -> Option<NodeId> {
		NodeId::new(self.0.nodeid)
	}

	pub fn set_id(&mut self, id: NodeId) {
		self.0.nodeid = id.0.get();
		self.0.attr.ino = id.0.get();
	}

	pub fn generation(&self) -> u64 {
		self.0.generation
	}

	pub fn set_generation(&mut self, generation: u64) {
		self.0.generation = generation;
	}

	pub fn kind(&self) -> NodeKind {
		let mode = self.0.attr.mode;
		NodeKind((mode >> 12) & 0xF)
	}

	pub fn set_kind(&mut self, kind: NodeKind) {
		let old_mode = self.0.attr.mode;
		let new_mode = (kind.0 & 0xF) << 12 | (old_mode & 0xFFF);
		self.0.attr.mode = new_mode;
	}

	pub fn cache_timeout(&self) -> time::Duration {
		time::Duration::new(self.0.entry_valid, self.0.entry_valid_nsec)
	}

	pub fn set_cache_timeout(&mut self, cache_timeout: time::Duration) {
		self.0.entry_valid = cache_timeout.as_secs();
		self.0.entry_valid_nsec = cache_timeout.subsec_nanos();
	}

	pub fn attr(&self) -> &NodeAttr {
		NodeAttr::new_ref(&self.0.attr)
	}

	pub fn attr_mut(&mut self) -> &mut NodeAttr {
		NodeAttr::new_ref_mut(&mut self.0.attr)
	}

	pub fn attr_cache_timeout(&self) -> time::Duration {
		time::Duration::new(self.0.attr_valid, self.0.attr_valid_nsec)
	}

	pub fn set_attr_cache_timeout(&mut self, d: time::Duration) {
		self.0.attr_valid = d.as_secs();
		self.0.attr_valid_nsec = d.subsec_nanos();
	}
}

impl fmt::Debug for Node {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Node")
			.field("id", &self.0.nodeid)
			.finish()
	}
}

// }}}
