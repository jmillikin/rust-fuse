// Copyright 2022 John Millikin and the rust-fuse contributors.
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

use core::fmt;

use crate::file_mode::FileMode;
use crate::kernel;
use crate::node_id::NodeId;

/// Attributes of a filesystem node.
#[derive(Clone, Copy)]
pub struct Attributes {
	pub(crate) raw: kernel::fuse_attr,
}

impl Attributes {
	/// Creates a new `Attributes` for a node with the given ID.
	#[inline]
	#[must_use]
	pub fn new(node_id: NodeId) -> Attributes {
		Self {
			raw: kernel::fuse_attr {
				ino: node_id.get(),
				..kernel::fuse_attr::new()
			},
		}
	}

	/// Returns the raw [`fuse_attr`] for the node attributes.
	///
	/// [`fuse_attr`]: kernel::fuse_attr
	#[inline]
	#[must_use]
	pub fn raw(&self) -> &kernel::fuse_attr {
		&self.raw
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref(raw: &kernel::fuse_attr) -> &Self {
		let raw_ptr = raw as *const kernel::fuse_attr;
		&*(raw_ptr.cast::<Attributes>())
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref_mut(
		raw: &mut kernel::fuse_attr,
	) -> &mut Self {
		let raw_ptr = raw as *mut kernel::fuse_attr;
		&mut *(raw_ptr.cast::<Attributes>())
	}

	/// Returns the per-mount unique identifier of the node.
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> NodeId {
		unsafe { NodeId::new_unchecked(self.raw.ino) }
	}

	/// Returns the node's mode, including type and permissions.
	#[inline]
	#[must_use]
	pub fn mode(&self) -> FileMode {
		FileMode::new(self.raw.mode)
	}

	/// Sets the node's mode, including type and permissions.
	#[inline]
	pub fn set_mode(&mut self, mode: FileMode) {
		self.raw.mode = mode.get();
	}

	/// Returns the node's size.
	#[inline]
	#[must_use]
	pub fn size(&self) -> u64 {
		self.raw.size
	}

	/// Sets the node's size.
	#[inline]
	pub fn set_size(&mut self, size: u64) {
		self.raw.size = size;
	}

	/// Returns the node's last access time.
	#[inline]
	#[must_use]
	pub fn atime(&self) -> crate::UnixTime {
		unsafe {
			crate::UnixTime::from_timespec_unchecked(
				self.raw.atime,
				self.raw.atimensec,
			)
		}
	}

	/// Sets the node's last access time.
	#[inline]
	pub fn set_atime(&mut self, atime: crate::UnixTime) {
		(self.raw.atime, self.raw.atimensec) = atime.as_timespec();
	}

	/// Returns the node's last content modification time.
	#[inline]
	#[must_use]
	pub fn mtime(&self) -> crate::UnixTime {
		unsafe {
			crate::UnixTime::from_timespec_unchecked(
				self.raw.mtime,
				self.raw.mtimensec,
			)
		}
	}

	/// Sets the node's last content modification time.
	#[inline]
	pub fn set_mtime(&mut self, mtime: crate::UnixTime) {
		(self.raw.mtime, self.raw.mtimensec) = mtime.as_timespec();
	}

	/// Returns the node's last status change time.
	#[inline]
	#[must_use]
	pub fn ctime(&self) -> crate::UnixTime {
		unsafe {
			crate::UnixTime::from_timespec_unchecked(
				self.raw.ctime,
				self.raw.ctimensec,
			)
		}
	}

	/// Sets the node's last status change time.
	#[inline]
	pub fn set_ctime(&mut self, ctime: crate::UnixTime) {
		(self.raw.ctime, self.raw.ctimensec) = ctime.as_timespec();
	}

	/// Returns the node's link count.
	#[inline]
	#[must_use]
	pub fn link_count(&self) -> u32 {
		self.raw.nlink
	}

	/// Sets the node's link count.
	///
	/// In general nodes accessible via `FUSE_LOOKUP` should have a non-zero
	/// link count. A link count of zero means the node has been removed but is
	/// still referenced by an open file handle.
	#[inline]
	pub fn set_link_count(&mut self, link_count: u32) {
		self.raw.nlink = link_count;
	}

	/// Returns the node's owning user ID.
	#[inline]
	#[must_use]
	pub fn user_id(&self) -> u32 {
		self.raw.uid
	}

	/// Sets the node's owning user ID.
	#[inline]
	pub fn set_user_id(&mut self, user_id: u32) {
		self.raw.uid = user_id;
	}

	/// Returns the node's owning group ID.
	#[inline]
	#[must_use]
	pub fn group_id(&self) -> u32 {
		self.raw.gid
	}

	/// Sets the node's owning group ID.
	#[inline]
	pub fn set_group_id(&mut self, group_id: u32) {
		self.raw.gid = group_id;
	}

	/// Returns the [device number] of a [`BlockDevice`] or [`CharacterDevice`]
	/// node.
	///
	/// [device number]: https://www.kernel.org/doc/html/latest/admin-guide/devices.html
	/// [`BlockDevice`]: crate::FileType::BlockDevice
	/// [`CharacterDevice`]: crate::FileType::CharacterDevice
	#[inline]
	#[must_use]
	pub fn device_number(&self) -> u32 {
		self.raw.rdev
	}

	/// Sets the [device number] of a [`BlockDevice`] or [`CharacterDevice`]
	/// node.
	///
	/// [device number]: https://www.kernel.org/doc/html/latest/admin-guide/devices.html
	/// [`BlockDevice`]: crate::FileType::BlockDevice
	/// [`CharacterDevice`]: crate::FileType::CharacterDevice
	#[inline]
	pub fn set_device_number(&mut self, device_number: u32) {
		self.raw.rdev = device_number;
	}

	/// Returns the number of blocks allocated by the node.
	#[inline]
	#[must_use]
	pub fn block_count(&self) -> u64 {
		self.raw.blocks
	}

	/// Sets the number of blocks allocated by the node.
	#[inline]
	pub fn set_block_count(&mut self, block_count: u64) {
		self.raw.blocks = block_count;
	}

	/// Returns the block size of the node.
	#[inline]
	#[must_use]
	pub fn block_size(&self) -> u32 {
		self.raw.blksize
	}

	/// Sets the block size of the node.
	#[inline]
	pub fn set_block_size(&mut self, block_size: u32) {
		self.raw.blksize = block_size;
	}

	/// Returns whether the node is the root of a submount.
	#[inline]
	#[must_use]
	pub fn flag_submount(&self) -> bool {
		self.flags().get(AttributeFlag::FUSE_ATTR_SUBMOUNT)
	}

	/// Sets whether the node is the root of a submount.
	#[inline]
	pub fn set_flag_submount(&mut self, is_submount: bool) {
		self.flags_mut().set_to(AttributeFlag::FUSE_ATTR_SUBMOUNT, is_submount)
	}

	/// Returns whether [DAX] is enabled for the node.
	///
	/// [DAX]: https://www.kernel.org/doc/html/latest/filesystems/dax.html
	#[inline]
	#[must_use]
	pub fn flag_dax(&self) -> bool {
		self.flags().get(AttributeFlag::FUSE_ATTR_DAX)
	}

	/// Sets whether [DAX] is enabled for the node.
	///
	/// [DAX]: https://www.kernel.org/doc/html/latest/filesystems/dax.html
	#[inline]
	pub fn set_flag_dax(&mut self, enable_dax: bool) {
		self.flags_mut().set_to(AttributeFlag::FUSE_ATTR_DAX, enable_dax)
	}

	#[inline]
	#[must_use]
	fn flags(&self) -> AttributeFlags {
		AttributeFlags {
			bits: self.raw.flags,
		}
	}

	#[inline]
	#[must_use]
	fn flags_mut(&mut self) -> &mut AttributeFlags {
		AttributeFlags::reborrow_mut(&mut self.raw.flags)
	}
}

impl fmt::Debug for Attributes {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Attributes")
			.field("node_id", &self.node_id())
			.field("mode", &self.mode())
			.field("size", &self.size())
			.field("atime", &format_args!("{:?}", self.atime()))
			.field("mtime", &format_args!("{:?}", self.mtime()))
			.field("ctime", &format_args!("{:?}", self.ctime()))
			.field("link_count", &self.link_count())
			.field("user_id", &self.user_id())
			.field("group_id", &self.group_id())
			.field("device_number", &self.device_number())
			.field("block_count", &self.block_count())
			.field("block_size", &self.block_size())
			.field("flags", &self.flags())
			.finish()
	}
}

/// Optional flags set on [`Attributes`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct AttributeFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct AttributeFlag {
	mask: u32,
}

mod attr_flags {
	use crate::kernel;
	bitflags!(AttributeFlag, AttributeFlags, u32, {
		FUSE_ATTR_SUBMOUNT = kernel::FUSE_ATTR_SUBMOUNT;
		FUSE_ATTR_DAX = kernel::FUSE_ATTR_DAX;
	});
}
