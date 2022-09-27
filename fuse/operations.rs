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

macro_rules! response_send_funcs {
	() => {
		pub fn send<S: crate::server::io::Socket>(
			&self,
			socket: &S,
			response_ctx: &crate::server::ResponseContext,
		) -> Result<(), crate::server::io::SendError<S::Error>> {
			use crate::server::io::encode::SyncSendOnce;
			let send = SyncSendOnce::new(socket);
			self.encode(send, response_ctx)
		}

		pub async fn send_async<S: crate::server::io::AsyncSocket>(
			&self,
			socket: &S,
			response_ctx: &crate::server::ResponseContext,
		) -> Result<(), crate::server::io::SendError<S::Error>> {
			use crate::server::io::encode::AsyncSendOnce;
			let send = AsyncSendOnce::new(socket);
			self.encode(send, response_ctx).await
		}
	};
	($t:ty) => {
		impl $t {
			response_send_funcs!();
		}
	};
}

pub mod access;
#[doc(inline)]
pub use self::access::*;

#[cfg(any(doc, feature = "unstable_bmap"))]
pub mod bmap;
#[cfg(any(doc, feature = "unstable_bmap"))]
#[doc(inline)]
pub use self::bmap::*;

pub mod copy_file_range;
#[doc(inline)]
pub use self::copy_file_range::*;

pub mod create;
#[doc(inline)]
pub use self::create::*;

pub mod cuse_init;
#[doc(inline)]
pub use self::cuse_init::*;

pub mod fallocate;
#[doc(inline)]
pub use self::fallocate::*;

pub mod flush;
#[doc(inline)]
pub use self::flush::*;

pub mod forget;
#[doc(inline)]
pub use self::forget::*;

pub mod fsync;
#[doc(inline)]
pub use self::fsync::*;

pub mod fsyncdir;
#[doc(inline)]
pub use self::fsyncdir::*;

pub mod fuse_init;
#[doc(inline)]
pub use self::fuse_init::*;

pub mod getattr;
#[doc(inline)]
pub use self::getattr::*;

pub mod getlk;
#[doc(inline)]
pub use self::getlk::*;

pub mod getxattr;
#[doc(inline)]
pub use self::getxattr::*;

pub mod ioctl;
#[doc(inline)]
pub use self::ioctl::*;

pub mod link;
#[doc(inline)]
pub use self::link::*;

pub mod listxattr;
#[doc(inline)]
pub use self::listxattr::*;

pub mod lookup;
#[doc(inline)]
pub use self::lookup::*;

pub mod lseek;
#[doc(inline)]
pub use self::lseek::*;

pub mod mkdir;
#[doc(inline)]
pub use self::mkdir::*;

pub mod mknod;
#[doc(inline)]
pub use self::mknod::*;

pub mod open;
#[doc(inline)]
pub use self::open::*;

pub mod opendir;
#[doc(inline)]
pub use self::opendir::*;

pub mod poll;
#[doc(inline)]
pub use self::poll::*;

pub mod read;
#[doc(inline)]
pub use self::read::*;

pub mod readdir;
#[doc(inline)]
pub use self::readdir::*;

pub mod readlink;
#[doc(inline)]
pub use self::readlink::*;

pub mod release;
#[doc(inline)]
pub use self::release::*;

pub mod releasedir;
#[doc(inline)]
pub use self::releasedir::*;

pub mod removexattr;
#[doc(inline)]
pub use self::removexattr::*;

pub mod rename;
#[doc(inline)]
pub use self::rename::*;

pub mod rmdir;
#[doc(inline)]
pub use self::rmdir::*;

pub mod setattr;
#[doc(inline)]
pub use self::setattr::*;

pub mod setlk;
#[doc(inline)]
pub use self::setlk::*;

pub mod setxattr;
#[doc(inline)]
pub use self::setxattr::*;

pub mod statfs;
#[doc(inline)]
pub use self::statfs::*;

pub mod symlink;
#[doc(inline)]
pub use self::symlink::*;

pub mod syncfs;
#[doc(inline)]
pub use self::syncfs::*;

pub mod unlink;
#[doc(inline)]
pub use self::unlink::*;

pub mod write;
#[doc(inline)]
pub use self::write::*;

pub(crate) mod types_only {
	mod access {}
	mod bmap {}
	mod copy_file_range {}
	mod create {}
	mod cuse_init {}
	mod fallocate {}
	mod flush {}
	mod forget {}
	mod fsync {}
	mod fsyncdir {}
	mod fuse_init {}
	mod getattr {}
	mod getlk {}
	mod getxattr {}
	mod ioctl {}
	mod link {}
	mod listxattr {}
	mod lookup {}
	mod lseek {}
	mod mkdir {}
	mod mknod {}
	mod open {}
	mod opendir {}
	mod read {}
	mod readdir {}
	mod readlink {}
	mod release {}
	mod releasedir {}
	mod removexattr {}
	mod rename {}
	mod rmdir {}
	mod setattr {}
	mod setlk {}

	pub use crate::operations::*;
}
