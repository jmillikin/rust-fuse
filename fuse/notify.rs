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

use core::convert::TryInto;
use core::fmt;
use core::num::NonZeroU64;

use crate::internal::fuse_kernel;
use crate::node;
use crate::server::encode;
use crate::server::io;

// FuseNotification {{{

#[non_exhaustive]
pub enum FuseNotification<'a> {
	Delete(Delete<'a>),
	InvalidateEntry(InvalidateEntry<'a>),
	InvalidateInode(InvalidateInode),
	Poll(Poll),
}

impl FuseNotification<'_> {
	pub fn send<S: io::Socket>(
		&self,
		socket: &S,
	) -> Result<(), io::SendError<S::Error>> {
		let send = encode::SyncSendOnce::new(socket);
		self.encode(send)
	}

	pub async fn send_async<S: io::AsyncSocket>(
		&self,
		socket: &S,
	) -> Result<(), io::SendError<S::Error>> {
		let send = encode::AsyncSendOnce::new(socket);
		self.encode(send).await
	}

	fn encode<S: encode::SendOnce>(&self, send: S) -> S::Result {
		let enc = encode::ReplyEncoder::new(send, 0);
		match self {
			FuseNotification::Poll(poll) => {
				enc.encode_unsolicited(
					fuse_kernel::FUSE_NOTIFY_POLL,
					&poll.raw,
					None,
				)
			},
			FuseNotification::InvalidateEntry(inval_entry) => {
				enc.encode_unsolicited(
					fuse_kernel::FUSE_NOTIFY_INVAL_ENTRY,
					&inval_entry.raw,
					Some(inval_entry.name.as_bytes()),
				)
			},
			FuseNotification::InvalidateInode(inval_inode) => {
				enc.encode_unsolicited(
					fuse_kernel::FUSE_NOTIFY_INVAL_INODE,
					&inval_inode.raw,
					None,
				)
			},
			FuseNotification::Delete(delete) => {
				enc.encode_unsolicited(
					fuse_kernel::FUSE_NOTIFY_DELETE,
					&delete.raw,
					Some(delete.name.as_bytes()),
				)
			},
		}
	}
}

// }}}

// Delete {{{

/// Notification message for `FUSE_NOTIFY_DELETE`.
pub struct Delete<'a> {
	raw: fuse_kernel::fuse_notify_delete_out,
	name: &'a node::Name,
}

impl<'a> Delete<'a> {
	#[must_use]
	pub fn new(
		parent_id: node::Id,
		node_id: node::Id,
		name: &'a node::Name,
	) -> Delete<'a> {
		Self {
			raw: fuse_kernel::fuse_notify_delete_out {
				parent: parent_id.get(),
				child: node_id.get(),
				namelen: name.as_bytes().len() as u32,
				padding: 0,
			},
			name,
		}
	}

	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.raw.parent) }
	}

	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.raw.child) }
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}
}

impl fmt::Debug for Delete<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Delete")
			.field("parent_id", &self.parent_id())
			.field("node_id", &self.node_id())
			.field("name", &self.name())
			.finish()
	}
}

// }}}

// InvalidateEntry {{{

/// Notification message for `FUSE_NOTIFY_INVAL_ENTRY`.
pub struct InvalidateEntry<'a> {
	raw: fuse_kernel::fuse_notify_inval_entry_out,
	name: &'a node::Name,
}

impl<'a> InvalidateEntry<'a> {
	#[must_use]
	pub fn new(
		parent_id: node::Id,
		name: &'a node::Name,
	) -> InvalidateEntry<'a> {
		Self {
			raw: fuse_kernel::fuse_notify_inval_entry_out {
				parent: parent_id.get(),
				namelen: name.as_bytes().len() as u32,
				padding: 0,
			},
			name,
		}
	}

	#[must_use]
	pub fn parent_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.raw.parent) }
	}

	#[must_use]
	pub fn name(&self) -> &node::Name {
		self.name
	}
}

impl fmt::Debug for InvalidateEntry<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("InvalidateEntry")
			.field("parent_id", &self.parent_id())
			.field("name", &self.name())
			.finish()
	}
}

// }}}

// InvalidateInode {{{

/// Notification message for `FUSE_NOTIFY_INVAL_INODE`.
pub struct InvalidateInode {
	raw: fuse_kernel::fuse_notify_inval_inode_out,
}

impl InvalidateInode {
	#[must_use]
	pub fn new(node_id: node::Id) -> InvalidateInode {
		Self {
			raw: fuse_kernel::fuse_notify_inval_inode_out {
				ino: node_id.get(),
				off: 0,
				len: 0,
			},
		}
	}

	#[must_use]
	pub fn node_id(&self) -> node::Id {
		unsafe { node::Id::new_unchecked(self.raw.ino) }
	}

	#[must_use]
	pub fn offset(&self) -> i64 {
		self.raw.off
	}

	pub fn set_offset(&mut self, offset: Option<u64>) {
		self.raw.off = match offset {
			Some(offset_u64) => match offset_u64.try_into() {
				Ok(offset_i64) => offset_i64,
				Err(_) => i64::MAX,
			},
			None => -1,
		};
	}

	#[must_use]
	pub fn size(&self) -> i64 {
		self.raw.len
	}

	pub fn set_size(&mut self, size: Option<NonZeroU64>) {
		self.raw.len = match size {
			Some(size_u64) => match size_u64.get().try_into() {
				Ok(size_i64) => size_i64,
				Err(_) => i64::MAX,
			},
			None => 0,
		};
	}
}

impl fmt::Debug for InvalidateInode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("InvalidateInode")
			.field("node_id", &self.node_id())
			.field("offset", &self.offset())
			.field("size", &self.size())
			.finish()
	}
}

// }}}

// Poll {{{

/// Notification message for `FUSE_NOTIFY_POLL`.
pub struct Poll {
	raw: fuse_kernel::fuse_notify_poll_wakeup_out,
}

impl Poll {
	#[must_use]
	pub fn new(poll_handle: crate::PollHandle) -> Poll {
		Self {
			raw: fuse_kernel::fuse_notify_poll_wakeup_out {
				kh: poll_handle.bits,
			},
		}
	}

	#[must_use]
	pub fn poll_handle(&self) -> crate::PollHandle {
		crate::PollHandle { bits: self.raw.kh }
	}
}

impl fmt::Debug for Poll {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Poll")
			.field("poll_handle", &self.poll_handle())
			.finish()
	}
}

// }}}
