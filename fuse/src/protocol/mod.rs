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

#![allow(unused_attributes)]

#[macro_use]
pub(crate) mod common;

pub use self::common::XattrError;

#[macro_use]
mod prelude;

#[path = "access/access.rs"]
pub mod access;
pub use self::access::*;

#[cfg(any(doc, feature = "unstable_bmap"))]
#[path = "bmap/bmap.rs"]
pub mod bmap;
#[cfg(any(doc, feature = "unstable_bmap"))]
pub use self::bmap::*;

#[path = "create/create.rs"]
pub mod create;
pub use self::create::*;

#[path = "cuse_init/cuse_init.rs"]
pub mod cuse_init;
pub use self::cuse_init::*;

#[path = "fallocate/fallocate.rs"]
pub mod fallocate;
pub use self::fallocate::*;

#[path = "flush/flush.rs"]
pub mod flush;
pub use self::flush::*;

#[path = "forget/forget.rs"]
pub mod forget;
pub use self::forget::*;

#[path = "fsync/fsync.rs"]
pub mod fsync;
pub use self::fsync::*;

#[path = "fsyncdir/fsyncdir.rs"]
pub mod fsyncdir;
pub use self::fsyncdir::*;

#[path = "fuse_init/fuse_init.rs"]
pub mod fuse_init;

#[path = "getattr/getattr.rs"]
pub mod getattr;
pub use self::getattr::*;

#[path = "getlk/getlk.rs"]
pub mod getlk;
pub use self::getlk::*;

#[path = "getxattr/getxattr.rs"]
pub mod getxattr;
pub use self::getxattr::*;

#[cfg(any(doc, feature = "unstable_ioctl"))]
#[path = "ioctl/ioctl.rs"]
pub mod ioctl;
#[cfg(any(doc, feature = "unstable_ioctl"))]
pub use self::ioctl::*;

#[path = "link/link.rs"]
pub mod link;
pub use self::link::*;

#[path = "listxattr/listxattr.rs"]
pub mod listxattr;
pub use self::listxattr::*;

#[path = "lookup/lookup.rs"]
pub mod lookup;
pub use self::lookup::*;

#[path = "lseek/lseek.rs"]
pub mod lseek;
pub use self::lseek::*;

#[path = "mkdir/mkdir.rs"]
pub mod mkdir;
pub use self::mkdir::*;

#[path = "mknod/mknod.rs"]
pub mod mknod;
pub use self::mknod::*;

#[path = "open/open.rs"]
pub mod open;
pub use self::open::*;

#[path = "opendir/opendir.rs"]
pub mod opendir;
pub use self::opendir::*;

#[path = "read/read.rs"]
pub mod read;
pub use self::read::*;

#[path = "readdir/readdir.rs"]
pub mod readdir;
pub use self::readdir::*;

#[path = "readlink/readlink.rs"]
pub mod readlink;
pub use self::readlink::*;

#[path = "release/release.rs"]
pub mod release;
pub use self::release::*;

#[path = "releasedir/releasedir.rs"]
pub mod releasedir;
pub use self::releasedir::*;

#[path = "removexattr/removexattr.rs"]
pub mod removexattr;
pub use self::removexattr::*;

#[path = "rename/rename.rs"]
pub mod rename;
pub use self::rename::*;

#[path = "rmdir/rmdir.rs"]
pub mod rmdir;
pub use self::rmdir::*;

#[cfg(any(doc, feature = "unstable_setattr"))]
#[path = "setattr/setattr.rs"]
pub mod setattr;
#[cfg(any(doc, feature = "unstable_setattr"))]
pub use self::setattr::*;

#[path = "setlk/setlk.rs"]
pub mod setlk;
pub use self::setlk::*;

#[path = "setxattr/setxattr.rs"]
pub mod setxattr;
pub use self::setxattr::*;

#[path = "statfs/statfs.rs"]
pub mod statfs;
pub use self::statfs::*;

#[path = "symlink/symlink.rs"]
pub mod symlink;
pub use self::symlink::*;

#[path = "unlink/unlink.rs"]
pub mod unlink;
pub use self::unlink::*;

#[path = "write/write.rs"]
pub mod write;
pub use self::write::*;
