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

pub use self::common::{
	Opcode,
	RequestHeader,
	ResponseHeader,
	UnknownRequest,
	XattrError,
};

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

op_mod!(access,      "access/access.rs",           "unstable_access");
op_mod!(bmap,        "bmap/bmap.rs",               "unstable_bmap");
op_mod!(create,      "create/create.rs",           "unstable_create");
op_mod!(cuse_init,   "cuse_init/cuse_init.rs");
op_mod!(fallocate,   "fallocate/fallocate.rs",     "unstable_fallocate");
op_mod!(flush,       "flush/flush.rs",             "unstable_flush");
op_mod!(forget,      "forget/forget.rs");
op_mod!(fsync,       "fsync/fsync.rs",             "unstable_fsync");
op_mod!(fsyncdir,    "fsyncdir/fsyncdir.rs",       "unstable_fsyncdir");
op_mod!(fuse_init,   "fuse_init/fuse_init.rs");
op_mod!(getattr,     "getattr/getattr.rs");
op_mod!(getlk,       "getlk/getlk.rs",             "unstable_getlk");
op_mod!(getxattr,    "getxattr/getxattr.rs");
op_mod!(ioctl,       "ioctl/ioctl.rs",             "unstable_ioctl");
op_mod!(link,        "link/link.rs");
op_mod!(listxattr,   "listxattr/listxattr.rs");
op_mod!(lookup,      "lookup/lookup.rs");
op_mod!(lseek,       "lseek/lseek.rs",             "unstable_lseek");
op_mod!(mkdir,       "mkdir/mkdir.rs");
op_mod!(mknod,       "mknod/mknod.rs");
op_mod!(open,        "open/open.rs");
op_mod!(opendir,     "opendir/opendir.rs");
op_mod!(read,        "read/read.rs");
op_mod!(readdir,     "readdir/readdir.rs");
op_mod!(readlink,    "readlink/readlink.rs");
op_mod!(release,     "release/release.rs");
op_mod!(releasedir,  "releasedir/releasedir.rs");
op_mod!(removexattr, "removexattr/removexattr.rs", "unstable_removexattr");
op_mod!(rename,      "rename/rename.rs");
op_mod!(rmdir,       "rmdir/rmdir.rs");
op_mod!(setattr,     "setattr/setattr.rs",         "unstable_setattr");
op_mod!(setlk,       "setlk/setlk.rs",             "unstable_setlk");
op_mod!(setxattr,    "setxattr/setxattr.rs",       "unstable_setxattr");
op_mod!(statfs,      "statfs/statfs.rs",           "unstable_statfs");
op_mod!(symlink,     "symlink/symlink.rs");
op_mod!(unlink,      "unlink/unlink.rs");
op_mod!(write,       "write/write.rs");
