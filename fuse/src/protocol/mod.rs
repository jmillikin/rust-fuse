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

#[macro_use]
mod common;

mod prelude;

#[cfg(any(doc, feature = "unstable_fuse_access"))]
#[doc(cfg(feature = "unstable_fuse_access"))]
#[path = "access/access.rs"]
mod access;
#[cfg(any(doc, feature = "unstable_fuse_access"))]
#[doc(cfg(feature = "unstable_fuse_access"))]
pub use self::access::*;

#[path = "bmap/bmap.rs"]
mod bmap;
pub use self::bmap::*;

#[path = "create/create.rs"]
mod create;
pub use self::create::*;

#[path = "fallocate/fallocate.rs"]
mod fallocate;
pub use self::fallocate::*;

#[path = "flush/flush.rs"]
mod flush;
pub use self::flush::*;

#[path = "forget/forget.rs"]
mod forget;
pub use self::forget::*;

#[path = "fsync/fsync.rs"]
mod fsync;
pub use self::fsync::*;

#[path = "fsyncdir/fsyncdir.rs"]
mod fsyncdir;
pub use self::fsyncdir::*;

#[path = "fuse_init/fuse_init.rs"]
mod fuse_init;
pub use self::fuse_init::*;

#[path = "getattr/getattr.rs"]
mod getattr;
pub use self::getattr::*;

#[path = "getlk/getlk.rs"]
mod getlk;
pub use self::getlk::*;

#[path = "getxattr/getxattr.rs"]
mod getxattr;
pub use self::getxattr::*;

#[path = "ioctl/ioctl.rs"]
mod ioctl;
pub use self::ioctl::*;

#[path = "link/link.rs"]
mod link;
pub use self::link::*;

#[path = "listxattr/listxattr.rs"]
mod listxattr;
pub use self::listxattr::*;

#[path = "lookup/lookup.rs"]
mod lookup;
pub use self::lookup::*;

#[path = "lseek/lseek.rs"]
mod lseek;
pub use self::lseek::*;

#[path = "mkdir/mkdir.rs"]
mod mkdir;
pub use self::mkdir::*;

#[path = "mknod/mknod.rs"]
mod mknod;
pub use self::mknod::*;

#[path = "open/open.rs"]
mod open;
pub use self::open::*;

#[path = "opendir/opendir.rs"]
mod opendir;
pub use self::opendir::*;

#[path = "read/read.rs"]
mod read;
pub use self::read::*;

#[path = "readdir/readdir.rs"]
mod readdir;
pub use self::readdir::*;

#[path = "readlink/readlink.rs"]
mod readlink;
pub use self::readlink::*;

#[path = "release/release.rs"]
mod release;
pub use self::release::*;

#[path = "releasedir/releasedir.rs"]
mod releasedir;
pub use self::releasedir::*;

#[path = "removexattr/removexattr.rs"]
mod removexattr;
pub use self::removexattr::*;

#[path = "rename/rename.rs"]
mod rename;
pub use self::rename::*;

#[path = "rmdir/rmdir.rs"]
mod rmdir;
pub use self::rmdir::*;

#[path = "setattr/setattr.rs"]
mod setattr;
pub use self::setattr::*;

#[path = "setlk/setlk.rs"]
mod setlk;
pub use self::setlk::*;

#[path = "setxattr/setxattr.rs"]
mod setxattr;
pub use self::setxattr::*;

#[path = "statfs/statfs.rs"]
mod statfs;
pub use self::statfs::*;

#[path = "symlink/symlink.rs"]
mod symlink;
pub use self::symlink::*;

#[path = "unlink/unlink.rs"]
mod unlink;
pub use self::unlink::*;

#[path = "write/write.rs"]
mod write;
pub use self::write::*;

mod node;
pub use self::node::{Node, NodeAttr, NodeId, NodeKind, NodeName};

#[path = "unknown/unknown.rs"]
mod unknown;
pub use self::unknown::*;
