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

#![allow(missing_docs)] // TODO

use core::convert::TryInto;
use core::fmt;
use core::mem;
use core::num;

use crate::internal::fuse_kernel;
use crate::operations::poll;
use crate::server::encode;
use crate::server::io;
#[cfg(feature = "unstable_async")]
use crate::server_async;

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
		let mut header = crate::ResponseHeader::new_notification();
		socket.send(self.encode(&mut header))
	}

	#[cfg(feature = "unstable_async")]
	pub async fn send_async<S: server_async::io::Socket>(
		&self,
		socket: &S,
	) -> Result<(), io::SendError<S::Error>> {
		let mut header = crate::ResponseHeader::new_notification();
		socket.send(self.encode(&mut header)).await
	}

	fn encode<'a>(
		&'a self,
		header: &'a mut crate::ResponseHeader,
	) -> crate::io::SendBuf<'a> {
		match self {
			FuseNotification::Poll(poll) => encode_notify(
				header,
				fuse_kernel::FUSE_NOTIFY_POLL,
				&poll.raw,
				None,
			),
			FuseNotification::InvalidateEntry(inval_entry) => encode_notify(
				header,
				fuse_kernel::FUSE_NOTIFY_INVAL_ENTRY,
				&inval_entry.raw,
				Some(inval_entry.name.as_bytes()),
			),
			FuseNotification::InvalidateInode(inval_inode) => encode_notify(
				header,
				fuse_kernel::FUSE_NOTIFY_INVAL_INODE,
				&inval_inode.raw,
				None,
			),
			FuseNotification::Delete(delete) => encode_notify(
				header,
				fuse_kernel::FUSE_NOTIFY_DELETE,
				&delete.raw,
				Some(delete.name.as_bytes()),
			),
		}
	}
}

fn encode_notify<'a, T: Sized>(
	header: &'a mut crate::ResponseHeader,
	notify_code: fuse_kernel::fuse_notify_code,
	body: &'a T,
	name_bytes: Option<&'a [u8]>,
) -> crate::io::SendBuf<'a> {
	let mut message_len = mem::size_of::<fuse_kernel::fuse_out_header>();
	message_len += mem::size_of::<T>();
	if let Some(name_bytes) = name_bytes {
		message_len += name_bytes.len();
		message_len += 1;
	}
	header.set_response_len(unsafe {
		num::NonZeroU32::new_unchecked(message_len as u32)
	});
	header.set_error(unsafe {
		num::NonZeroI32::new_unchecked(notify_code.0 as i32)
	});
	if let Some(name_bytes) = name_bytes {
		crate::io::SendBuf::new_4(
			message_len,
			encode::sized_to_slice(header),
			encode::sized_to_slice(body),
			name_bytes,
			b"\0",
		)
	} else {
		crate::io::SendBuf::new_2(
			message_len,
			encode::sized_to_slice(header),
			encode::sized_to_slice(body),
		)
	}
}

// }}}

// Delete {{{

/// Notification message for `FUSE_NOTIFY_DELETE`.
pub struct Delete<'a> {
	raw: fuse_kernel::fuse_notify_delete_out,
	name: &'a crate::NodeName,
}

impl<'a> Delete<'a> {
	#[must_use]
	pub fn new(
		parent_id: crate::NodeId,
		node_id: crate::NodeId,
		name: &'a crate::NodeName,
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
	pub fn parent_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.raw.parent) }
	}

	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.raw.child) }
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
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
	name: &'a crate::NodeName,
}

impl<'a> InvalidateEntry<'a> {
	#[must_use]
	pub fn new(
		parent_id: crate::NodeId,
		name: &'a crate::NodeName,
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
	pub fn parent_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.raw.parent) }
	}

	#[must_use]
	pub fn name(&self) -> &crate::NodeName {
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
	pub fn new(node_id: crate::NodeId) -> InvalidateInode {
		Self {
			raw: fuse_kernel::fuse_notify_inval_inode_out {
				ino: node_id.get(),
				off: 0,
				len: 0,
			},
		}
	}

	#[must_use]
	pub fn node_id(&self) -> crate::NodeId {
		unsafe { crate::NodeId::new_unchecked(self.raw.ino) }
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

	pub fn set_size(&mut self, size: Option<num::NonZeroU64>) {
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
	pub fn new(poll_handle: poll::PollHandle) -> Poll {
		Self {
			raw: fuse_kernel::fuse_notify_poll_wakeup_out {
				kh: poll_handle.bits,
			},
		}
	}

	#[must_use]
	pub fn poll_handle(&self) -> poll::PollHandle {
		poll::PollHandle { bits: self.raw.kh }
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
