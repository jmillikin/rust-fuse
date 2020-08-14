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

use core::{fmt, time};

use crate::internal::fuse_kernel;
use crate::protocol::common::NodeId;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NodeAttr(fuse_kernel::fuse_attr);

impl NodeAttr {
	pub fn node_id(&self) -> Option<NodeId> {
		NodeId::new(self.0.ino)
	}

	pub fn set_node_id(&mut self, node_id: NodeId) {
		self.0.ino = node_id.get();
	}

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

	pub fn mode(&self) -> u32 {
		self.0.mode
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
			.field("node_id", &format_args!("{:?}", &self.node_id()))
			.field("size", &self.0.size)
			.field("blocks", &self.0.blocks)
			.field("atime", &self.atime())
			.field("mtime", &self.mtime())
			.field("ctime", &self.ctime())
			.field("mode", &format_args!("{:#o}", &self.0.mode))
			.field("nlink", &self.0.nlink)
			.field("uid", &self.0.uid)
			.field("gid", &self.0.gid)
			.field("rdev", &self.0.rdev)
			.field("blksize", &self.0.blksize)
			.finish()
	}
}
