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
#![rustfmt::skip]

#[macro_use]
pub(crate) mod common;

mod prelude;

macro_rules! op_mod {
	($name:ident, $path:expr) => {
		#[path = $path]
		pub mod $name;
		pub use self::$name::*;
	};
	($name:ident, $path:expr, $feature:expr) => {
		#[cfg(any(doc, feature = $feature))]
		#[cfg_attr(doc, doc(cfg(feature = $feature)))]
		#[path = $path]
		pub mod $name;
		#[cfg(any(doc, feature = $feature))]
		pub use self::$name::*;
	};
}

op_mod!(access,      "access/access.rs",           "unstable_fuse_access");
op_mod!(bmap,        "bmap/bmap.rs",               "unstable_fuse_bmap");
op_mod!(create,      "create/create.rs",           "unstable_fuse_create");
op_mod!(fallocate,   "fallocate/fallocate.rs",     "unstable_fuse_fallocate");
op_mod!(flush,       "flush/flush.rs",             "unstable_fuse_flush");
op_mod!(forget,      "forget/forget.rs");
op_mod!(fsync,       "fsync/fsync.rs",             "unstable_fuse_fsync");
op_mod!(fsyncdir,    "fsyncdir/fsyncdir.rs",       "unstable_fuse_fsyncdir");
op_mod!(fuse_init,   "fuse_init/fuse_init.rs");
op_mod!(getattr,     "getattr/getattr.rs");
op_mod!(getlk,       "getlk/getlk.rs",             "unstable_fuse_getlk");
op_mod!(getxattr,    "getxattr/getxattr.rs");
op_mod!(ioctl,       "ioctl/ioctl.rs",             "unstable_fuse_ioctl");
op_mod!(link,        "link/link.rs");
op_mod!(listxattr,   "listxattr/listxattr.rs");
op_mod!(lookup,      "lookup/lookup.rs");
op_mod!(lseek,       "lseek/lseek.rs",             "unstable_fuse_lseek");
op_mod!(mkdir,       "mkdir/mkdir.rs");
op_mod!(mknod,       "mknod/mknod.rs",             "unstable_fuse_mknod");
op_mod!(open,        "open/open.rs");
op_mod!(opendir,     "opendir/opendir.rs");
op_mod!(read,        "read/read.rs");
op_mod!(readdir,     "readdir/readdir.rs");
op_mod!(readlink,    "readlink/readlink.rs");
op_mod!(release,     "release/release.rs");
op_mod!(releasedir,  "releasedir/releasedir.rs");
op_mod!(removexattr, "removexattr/removexattr.rs", "unstable_fuse_removexattr");
op_mod!(rename,      "rename/rename.rs",           "unstable_fuse_rename");
op_mod!(rmdir,       "rmdir/rmdir.rs",             "unstable_fuse_rmdir");
op_mod!(setattr,     "setattr/setattr.rs",         "unstable_fuse_setattr");
op_mod!(setlk,       "setlk/setlk.rs",             "unstable_fuse_setlk");
op_mod!(setxattr,    "setxattr/setxattr.rs",       "unstable_fuse_setxattr");
op_mod!(statfs,      "statfs/statfs.rs",           "unstable_fuse_statfs");
op_mod!(symlink,     "symlink/symlink.rs",         "unstable_fuse_symlink");
op_mod!(unknown,     "unknown/unknown.rs");
op_mod!(unlink,      "unlink/unlink.rs",           "unstable_fuse_unlink");
op_mod!(write,       "write/write.rs",             "unstable_fuse_write");
