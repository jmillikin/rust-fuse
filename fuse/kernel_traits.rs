// Copyright 2024 John Millikin and the rust-fuse contributors.
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

use crate::kernel;
use crate::server;

macro_rules! cuse_reply_sized {
	($t:ty) => {
		impl server::CuseReply for $t {
			fn send_to<S: server::CuseSocket>(
				&self,
				reply_sender: server::CuseReplySender<'_, S>,
			) -> Result<(), server::SendError<S::Error>> {
				reply_sender.inner.send_1(self.as_bytes())
			}
		}
	};
}

macro_rules! fuse_reply_sized {
	($t:ty) => {
		impl server::FuseReply for $t {
			fn send_to<S: server::FuseSocket>(
				&self,
				reply_sender: server::FuseReplySender<'_, S>,
			) -> Result<(), server::SendError<S::Error>> {
				reply_sender.inner.send_1(self.as_bytes())
			}
		}
	};
}

fuse_reply_sized!(kernel::fuse_bmap_out);
fuse_reply_sized!(kernel::fuse_getxattr_out);
fuse_reply_sized!(kernel::fuse_lk_out);
fuse_reply_sized!(kernel::fuse_lseek_out);
fuse_reply_sized!(kernel::fuse_open_out);
fuse_reply_sized!(kernel::fuse_poll_out);
fuse_reply_sized!(kernel::fuse_write_out);

cuse_reply_sized!(kernel::fuse_open_out);
cuse_reply_sized!(kernel::fuse_write_out);

impl server::FuseReply for kernel::fuse_attr_out {
	fn send_to<S: server::FuseSocket>(
		&self,
		reply_sender: server::FuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		let mut buf = self.as_bytes();
		if reply_sender.layout.version_minor() < 9 {
			buf = &buf[..kernel::FUSE_COMPAT_ATTR_OUT_SIZE];
		}
		reply_sender.inner.send_1(buf)
	}
}

impl server::FuseReply for kernel::fuse_entry_out {
	fn send_to<S: server::FuseSocket>(
		&self,
		reply_sender: server::FuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		let mut buf = self.as_bytes();
		if reply_sender.layout.version_minor() < 9 {
			buf = &buf[..kernel::FUSE_COMPAT_ENTRY_OUT_SIZE];
		}
		reply_sender.inner.send_1(buf)
	}
}

impl server::FuseReply for kernel::fuse_init_out {
	fn send_to<S: server::FuseSocket>(
		&self,
		reply_sender: server::FuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		let mut buf = self.as_bytes();
		if self.minor < 5 {
			buf = &buf[..kernel::FUSE_COMPAT_INIT_OUT_SIZE];
		} else if self.minor < 23 {
			buf = &buf[..kernel::FUSE_COMPAT_22_INIT_OUT_SIZE];
		}
		reply_sender.inner.send_1(buf)
	}
}

impl server::FuseReply for kernel::fuse_statfs_out {
	fn send_to<S: server::FuseSocket>(
		&self,
		reply_sender: server::FuseReplySender<'_, S>,
	) -> Result<(), server::SendError<S::Error>> {
		let mut buf = self.as_bytes();
		if reply_sender.layout.version_minor() < 4 {
			buf = &buf[..kernel::FUSE_COMPAT_STATFS_SIZE];
		}
		reply_sender.inner.send_1(buf)
	}
}
