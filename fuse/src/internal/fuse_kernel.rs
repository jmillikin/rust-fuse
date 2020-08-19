/* SPDX-License-Identifier: ((GPL-2.0 WITH Linux-syscall-note) OR BSD-2-Clause) */
/*
    This file defines the kernel interface of FUSE
    Copyright (C) 2001-2008  Miklos Szeredi <miklos@szeredi.hu>

    This program can be distributed under the terms of the GNU GPL.
    See the file COPYING.

    This -- and only this -- header file may also be distributed under
    the terms of the BSD Licence as follows:

    Copyright (C) 2001-2007 Miklos Szeredi. All rights reserved.

    Redistribution and use in source and binary forms, with or without
    modification, are permitted provided that the following conditions
    are met:
    1. Redistributions of source code must retain the above copyright
       notice, this list of conditions and the following disclaimer.
    2. Redistributions in binary form must reproduce the above copyright
       notice, this list of conditions and the following disclaimer in the
       documentation and/or other materials provided with the distribution.

    THIS SOFTWARE IS PROVIDED BY AUTHOR AND CONTRIBUTORS ``AS IS'' AND
    ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
    IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
    ARE DISCLAIMED.  IN NO EVENT SHALL AUTHOR OR CONTRIBUTORS BE LIABLE
    FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
    DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
    OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
    HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
    LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
    OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
    SUCH DAMAGE.
*/

/*
 * This file defines the kernel interface of FUSE
 *
 * Protocol changelog:
 *
 * 7.9:
 *  - new fuse_getattr_in input argument of GETATTR
 *  - add lk_flags in fuse_lk_in
 *  - add lock_owner field to fuse_setattr_in, fuse_read_in and fuse_write_in
 *  - add blksize field to fuse_attr
 *  - add file flags field to fuse_read_in and fuse_write_in
 *
 * 7.10
 *  - add nonseekable open flag
 *
 * 7.11
 *  - add IOCTL message
 *  - add unsolicited notification support
 *  - add POLL message and NOTIFY_POLL notification
 *
 * 7.12
 *  - add umask flag to input argument of open, mknod and mkdir
 *  - add notification messages for invalidation of inodes and
 *    directory entries
 *
 * 7.13
 *  - make max number of background requests and congestion threshold
 *    tunables
 *
 * 7.14
 *  - add splice support to fuse device
 *
 * 7.15
 *  - add store notify
 *  - add retrieve notify
 *
 * 7.16
 *  - add BATCH_FORGET request
 *  - FUSE_IOCTL_UNRESTRICTED shall now return with array of 'struct
 *    fuse_ioctl_iovec' instead of ambiguous 'struct iovec'
 *  - add FUSE_IOCTL_32BIT flag
 *
 * 7.17
 *  - add FUSE_FLOCK_LOCKS and FUSE_RELEASE_FLOCK_UNLOCK
 *
 * 7.18
 *  - add FUSE_IOCTL_DIR flag
 *  - add FUSE_NOTIFY_DELETE
 *
 * 7.19
 *  - add FUSE_FALLOCATE
 *
 * 7.20
 *  - add FUSE_AUTO_INVAL_DATA
 *
 * 7.21
 *  - add FUSE_READDIRPLUS
 *  - send the requested events in POLL request
 *
 * 7.22
 *  - add FUSE_ASYNC_DIO
 *
 * 7.23
 *  - add FUSE_WRITEBACK_CACHE
 *  - add time_gran to fuse_init_out
 *  - add reserved space to fuse_init_out
 *  - add FATTR_CTIME
 *  - add ctime and ctimensec to fuse_setattr_in
 *  - add FUSE_RENAME2 request
 *  - add FUSE_NO_OPEN_SUPPORT flag
 *
 *  7.24
 *  - add FUSE_LSEEK for SEEK_HOLE and SEEK_DATA support
 *
 *  7.25
 *  - add FUSE_PARALLEL_DIROPS
 *
 *  7.26
 *  - add FUSE_HANDLE_KILLPRIV
 *  - add FUSE_POSIX_ACL
 *
 *  7.27
 *  - add FUSE_ABORT_ERROR
 */

/*
 * Version negotiation:
 *
 * Both the kernel and userspace send the version they support in the
 * INIT request and reply respectively.
 *
 * If the major versions match then both shall use the smallest
 * of the two minor versions for communication.
 *
 * If the kernel supports a larger major version, then userspace shall
 * reply with the major version it supports, ignore the rest of the
 * INIT message and expect a new INIT message from the kernel with a
 * matching major version.
 *
 * If the library supports a larger major version, then it shall fall
 * back to the major protocol version sent by the kernel for
 * communication and reply with that major version (and an arbitrary
 * supported minor version).
 */

/** Version number of this interface */
pub const FUSE_KERNEL_VERSION: u32 = 7;

/** Minor version number of this interface */
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 27;

/** The node ID of the root inode */
pub const FUSE_ROOT_ID: u64 = 1;

/* Make sure all structures are padded to 64bit boundary, so 32bit
   userspace works under 64bit kernels */

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_attr {
	pub ino:       u64,
	pub size:      u64,
	pub blocks:    u64,
	pub atime:     u64,
	pub mtime:     u64,
	pub ctime:     u64,
	pub atimensec: u32,
	pub mtimensec: u32,
	pub ctimensec: u32,
	pub mode:      u32,
	pub nlink:     u32,
	pub uid:       u32,
	pub gid:       u32,
	pub rdev:      u32,
	pub blksize:   u32,
	pub padding:   u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_kstatfs {
	pub blocks:  u64,
	pub bfree:   u64,
	pub bavail:  u64,
	pub files:   u64,
	pub ffree:   u64,
	pub bsize:   u32,
	pub namelen: u32,
	pub frsize:  u32,
	pub padding: u32,
	pub spare:   [u32; 6],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_file_lock {
	pub start:  u64,
	pub end:    u64,
	pub r#type: u32,
	pub pid:    u32, /* tgid */
}

/**
 * Bitmasks for fuse_setattr_in.valid
 */
pub const FATTR_MODE:      u32 = 1 <<  0;
pub const FATTR_UID:       u32 = 1 <<  1;
pub const FATTR_GID:       u32 = 1 <<  2;
pub const FATTR_SIZE:      u32 = 1 <<  3;
pub const FATTR_ATIME:     u32 = 1 <<  4;
pub const FATTR_MTIME:     u32 = 1 <<  5;
pub const FATTR_FH:        u32 = 1 <<  6;
pub const FATTR_ATIME_NOW: u32 = 1 <<  7;
pub const FATTR_MTIME_NOW: u32 = 1 <<  8;
pub const FATTR_LOCKOWNER: u32 = 1 <<  9;
pub const FATTR_CTIME:     u32 = 1 << 10;

/**
 * Flags returned by the OPEN request
 *
 * FOPEN_DIRECT_IO: bypass page cache for this open file
 * FOPEN_KEEP_CACHE: don't invalidate the data cache on open
 * FOPEN_NONSEEKABLE: the file is not seekable
 */
pub const FOPEN_DIRECT_IO:   u32 = 1 <<  0;
pub const FOPEN_KEEP_CACHE:  u32 = 1 <<  1;
pub const FOPEN_NONSEEKABLE: u32 = 1 <<  2;

/**
 * INIT request/reply flags
 *
 * FUSE_ASYNC_READ: asynchronous read requests
 * FUSE_POSIX_LOCKS: remote locking for POSIX file locks
 * FUSE_FILE_OPS: kernel sends file handle for fstat, etc... (not yet supported)
 * FUSE_ATOMIC_O_TRUNC: handles the O_TRUNC open flag in the filesystem
 * FUSE_EXPORT_SUPPORT: filesystem handles lookups of "." and ".."
 * FUSE_BIG_WRITES: filesystem can handle write size larger than 4kB
 * FUSE_DONT_MASK: don't apply umask to file mode on create operations
 * FUSE_SPLICE_WRITE: kernel supports splice write on the device
 * FUSE_SPLICE_MOVE: kernel supports splice move on the device
 * FUSE_SPLICE_READ: kernel supports splice read on the device
 * FUSE_FLOCK_LOCKS: remote locking for BSD style file locks
 * FUSE_HAS_IOCTL_DIR: kernel supports ioctl on directories
 * FUSE_AUTO_INVAL_DATA: automatically invalidate cached pages
 * FUSE_DO_READDIRPLUS: do READDIRPLUS (READDIR+LOOKUP in one)
 * FUSE_READDIRPLUS_AUTO: adaptive readdirplus
 * FUSE_ASYNC_DIO: asynchronous direct I/O submission
 * FUSE_WRITEBACK_CACHE: use writeback cache for buffered writes
 * FUSE_NO_OPEN_SUPPORT: kernel supports zero-message opens
 * FUSE_PARALLEL_DIROPS: allow parallel lookups and readdir
 * FUSE_HANDLE_KILLPRIV: fs handles killing suid/sgid/cap on write/chown/trunc
 * FUSE_POSIX_ACL: filesystem supports posix acls
 * FUSE_ABORT_ERROR: reading the device after abort returns ECONNABORTED
 */
pub const FUSE_ASYNC_READ:       u32 = 1 <<  0;
pub const FUSE_POSIX_LOCKS:      u32 = 1 <<  1;
pub const FUSE_FILE_OPS:         u32 = 1 <<  2;
pub const FUSE_ATOMIC_O_TRUNC:   u32 = 1 <<  3;
pub const FUSE_EXPORT_SUPPORT:   u32 = 1 <<  4;
pub const FUSE_BIG_WRITES:       u32 = 1 <<  5;
pub const FUSE_DONT_MASK:        u32 = 1 <<  6;
pub const FUSE_SPLICE_WRITE:     u32 = 1 <<  7;
pub const FUSE_SPLICE_MOVE:      u32 = 1 <<  8;
pub const FUSE_SPLICE_READ:      u32 = 1 <<  9;
pub const FUSE_FLOCK_LOCKS:      u32 = 1 << 10;
pub const FUSE_HAS_IOCTL_DIR:    u32 = 1 << 11;
pub const FUSE_AUTO_INVAL_DATA:  u32 = 1 << 12;
pub const FUSE_DO_READDIRPLUS:   u32 = 1 << 13;
pub const FUSE_READDIRPLUS_AUTO: u32 = 1 << 14;
pub const FUSE_ASYNC_DIO:        u32 = 1 << 15;
pub const FUSE_WRITEBACK_CACHE:  u32 = 1 << 16;
pub const FUSE_NO_OPEN_SUPPORT:  u32 = 1 << 17;
pub const FUSE_PARALLEL_DIROPS:  u32 = 1 << 18;
pub const FUSE_HANDLE_KILLPRIV:  u32 = 1 << 19;
pub const FUSE_POSIX_ACL:        u32 = 1 << 20;
pub const FUSE_ABORT_ERROR:      u32 = 1 << 21;

/**
 * CUSE INIT request/reply flags
 *
 * CUSE_UNRESTRICTED_IOCTL:  use unrestricted ioctl
 */
pub const CUSE_UNRESTRICTED_IOCTL: u32 = 1 << 0;

/**
 * Release flags
 */
pub const FUSE_RELEASE_FLUSH:        u32 = 1 << 0;
pub const FUSE_RELEASE_FLOCK_UNLOCK: u32 = 1 << 1;

/**
 * Getattr flags
 */
pub const FUSE_GETATTR_FH: u32 = 1 << 0;

/**
 * Lock flags
 */
pub const FUSE_LK_FLOCK: u32 = 1 << 0;

/**
 * WRITE flags
 *
 * FUSE_WRITE_CACHE: delayed write from page cache, file handle is guessed
 * FUSE_WRITE_LOCKOWNER: lock_owner field is valid
 */
pub const FUSE_WRITE_CACHE:     u32 = 1 << 0;
pub const FUSE_WRITE_LOCKOWNER: u32 = 1 << 1;

/**
 * Read flags
 */
pub const FUSE_READ_LOCKOWNER: u32 = 1 << 1;

/**
 * Ioctl flags
 *
 * FUSE_IOCTL_COMPAT: 32bit compat ioctl on 64bit machine
 * FUSE_IOCTL_UNRESTRICTED: not restricted to well-formed ioctls, retry allowed
 * FUSE_IOCTL_RETRY: retry with new iovecs
 * FUSE_IOCTL_32BIT: 32bit ioctl
 * FUSE_IOCTL_DIR: is a directory
 *
 * FUSE_IOCTL_MAX_IOV: maximum of in_iovecs + out_iovecs
 */
pub const FUSE_IOCTL_COMPAT:       u32 = 1 << 0;
pub const FUSE_IOCTL_UNRESTRICTED: u32 = 1 << 1;
pub const FUSE_IOCTL_RETRY:        u32 = 1 << 2;
pub const FUSE_IOCTL_32BIT:        u32 = 1 << 3;
pub const FUSE_IOCTL_DIR:          u32 = 1 << 4;

pub const FUSE_IOCTL_MAX_IOV: u32 = 256;

/**
 * Poll flags
 *
 * FUSE_POLL_SCHEDULE_NOTIFY: request poll notify
 */
pub const FUSE_POLL_SCHEDULE_NOTIFY: u32 = 1 << 0;

enum_fuse_opcode! {
	FUSE_LOOKUP       = 1,
	FUSE_FORGET       = 2,  /* no reply */
	FUSE_GETATTR      = 3,
	FUSE_SETATTR      = 4,
	FUSE_READLINK     = 5,
	FUSE_SYMLINK      = 6,
	FUSE_MKNOD        = 8,
	FUSE_MKDIR        = 9,
	FUSE_UNLINK       = 10,
	FUSE_RMDIR        = 11,
	FUSE_RENAME       = 12,
	FUSE_LINK         = 13,
	FUSE_OPEN         = 14,
	FUSE_READ         = 15,
	FUSE_WRITE        = 16,
	FUSE_STATFS       = 17,
	FUSE_RELEASE      = 18,
	FUSE_FSYNC        = 20,
	FUSE_SETXATTR     = 21,
	FUSE_GETXATTR     = 22,
	FUSE_LISTXATTR    = 23,
	FUSE_REMOVEXATTR  = 24,
	FUSE_FLUSH        = 25,
	FUSE_INIT         = 26,
	FUSE_OPENDIR      = 27,
	FUSE_READDIR      = 28,
	FUSE_RELEASEDIR   = 29,
	FUSE_FSYNCDIR     = 30,
	FUSE_GETLK        = 31,
	FUSE_SETLK        = 32,
	FUSE_SETLKW       = 33,
	FUSE_ACCESS       = 34,
	FUSE_CREATE       = 35,
	FUSE_INTERRUPT    = 36,
	FUSE_BMAP         = 37,
	FUSE_DESTROY      = 38,
	FUSE_IOCTL        = 39,
	FUSE_POLL         = 40,
	FUSE_NOTIFY_REPLY = 41,
	FUSE_BATCH_FORGET = 42,
	FUSE_FALLOCATE    = 43,
	FUSE_READDIRPLUS  = 44,
	FUSE_RENAME2      = 45,
	FUSE_LSEEK        = 46,

	/* CUSE specific operations */
	CUSE_INIT         = 4096,
}

pub const FUSE_NOTIFY_POLL:        u32 = 1;
pub const FUSE_NOTIFY_INVAL_INODE: u32 = 2;
pub const FUSE_NOTIFY_INVAL_ENTRY: u32 = 3;
pub const FUSE_NOTIFY_STORE:       u32 = 4;
pub const FUSE_NOTIFY_RETRIEVE:    u32 = 5;
pub const FUSE_NOTIFY_DELETE:      u32 = 6;
pub const FUSE_NOTIFY_CODE_MAX:    u32 = 7;

/* The read buffer is required to be at least 8k, but may be much larger */
pub const FUSE_MIN_READ_BUFFER: usize = 8192;

pub const FUSE_COMPAT_ENTRY_OUT_SIZE: usize = 120;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_entry_out {
	pub nodeid:           u64, /* Inode ID */
	pub generation:       u64, /* Inode generation: nodeid:gen must
	                              be unique for the fs's lifetime */
	pub entry_valid:      u64, /* Cache timeout for the name */
	pub attr_valid:       u64, /* Cache timeout for the attributes */
	pub entry_valid_nsec: u32,
	pub attr_valid_nsec:  u32,
	pub attr:             fuse_attr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_forget_in {
	pub nlookup: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_forget_one {
	pub nodeid:  u64,
	pub nlookup: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_batch_forget_in {
	pub count: u32,
	pub dummy: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_getattr_in {
	pub getattr_flags: u32,
	pub dummy:         u32,
	pub fh:            u64,
}

pub const FUSE_COMPAT_ATTR_OUT_SIZE: usize = 96;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_attr_out {
	pub attr_valid:      u64, /* Cache timeout for the attributes */
	pub attr_valid_nsec: u32,
	pub dummy:           u32,
	pub attr:            fuse_attr,
}

pub const FUSE_COMPAT_MKNOD_IN_SIZE: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_mknod_in {
	pub mode:    u32,
	pub rdev:    u32,
	pub umask:   u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_mkdir_in {
	pub mode:  u32,
	pub umask: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_rename_in {
	pub newdir: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_rename2_in {
	pub newdir:  u64,
	pub flags:   u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_link_in {
	pub oldnodeid: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_setattr_in {
	pub valid:      u32,
	pub padding:    u32,
	pub fh:         u64,
	pub size:       u64,
	pub lock_owner: u64,
	pub atime:      u64,
	pub mtime:      u64,
	pub ctime:      u64,
	pub atimensec:  u32,
	pub mtimensec:  u32,
	pub ctimensec:  u32,
	pub mode:       u32,
	pub unused4:    u32,
	pub uid:        u32,
	pub gid:        u32,
	pub unused5:    u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_open_in {
	pub flags:  u32,
	pub unused: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_create_in {
	pub flags:   u32,
	pub mode:    u32,
	pub umask:   u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_open_out {
	pub fh:         u64,
	pub open_flags: u32,
	pub padding:    u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_release_in {
	pub fh:            u64,
	pub flags:         u32,
	pub release_flags: u32,
	pub lock_owner:    u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_flush_in {
	pub fh:         u64,
	pub unused:     u32,
	pub padding:    u32,
	pub lock_owner: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_read_in {
	pub fh:         u64,
	pub offset:     u64,
	pub size:       u32,
	pub read_flags: u32,
	pub lock_owner: u64,
	pub flags:      u32,
	pub padding:    u32,
}

pub const FUSE_COMPAT_WRITE_IN_SIZE: usize = 24;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_write_in {
	pub fh:          u64,
	pub offset:      u64,
	pub size:        u32,
	pub write_flags: u32,
	pub lock_owner:  u64,
	pub flags:       u32,
	pub padding:     u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_write_out {
	pub size:    u32,
	pub padding: u32,
}

pub const FUSE_COMPAT_STATFS_SIZE: usize = 48;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_statfs_out {
	pub st: fuse_kstatfs,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_fsync_in {
	pub fh:          u64,
	pub fsync_flags: u32,
	pub padding:     u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_setxattr_in {
	pub size:  u32,
	pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_getxattr_in {
	pub size:    u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_getxattr_out {
	pub size:    u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_lk_in {
	pub fh:       u64,
	pub owner:    u64,
	pub lk:       fuse_file_lock,
	pub lk_flags: u32,
	pub padding:  u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_lk_out {
	pub lk: fuse_file_lock,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_access_in {
	pub mask:    u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_init_in {
	pub major:         u32,
	pub minor:         u32,
	pub max_readahead: u32,
	pub flags:         u32,
}

pub const FUSE_COMPAT_INIT_OUT_SIZE:    usize =  8;
pub const FUSE_COMPAT_22_INIT_OUT_SIZE: usize = 24;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_init_out {
	pub major:                u32,
	pub minor:                u32,
	pub max_readahead:        u32,
	pub flags:                u32,
	pub max_background:       u16,
	pub congestion_threshold: u16,
	pub max_write:            u32,
	pub time_gran:            u32,
	pub unused:               [u32; 9],
}

pub const CUSE_INIT_INFO_MAX: u32 = 4096;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cuse_init_in {
	pub major:  u32,
	pub minor:  u32,
	pub unused: u32,
	pub flags:  u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cuse_init_out {
	pub major:     u32,
	pub minor:     u32,
	pub unused:    u32,
	pub flags:     u32,
	pub max_read:  u32,
	pub max_write: u32,
	pub dev_major: u32, /* chardev major */
	pub dev_minor: u32, /* chardev minor */
	pub spare:     [u32; 10],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_interrupt_in {
	pub unique: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_bmap_in {
	pub block:     u64,
	pub blocksize: u32,
	pub padding:   u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_bmap_out {
	pub block: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_ioctl_in {
	pub fh:       u64,
	pub flags:    u32,
	pub cmd:      u32,
	pub arg:      u64,
	pub in_size:  u32,
	pub out_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_ioctl_iovec {
	pub base: u64,
	pub len:  u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_ioctl_out {
	pub result:   i32,
	pub flags:    u32,
	pub io_iovs:  u32,
	pub out_iovs: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_poll_in {
	pub fh:     u64,
	pub kh:     u64,
	pub flags:  u32,
	pub events: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_poll_out {
	pub revents: u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_poll_wakeup_out {
	pub kh: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct fuse_fallocate_in {
	pub fh:      u64,
	pub offset:  u64,
	pub length:  u64,
	pub mode:    u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_in_header {
	pub len:     u32,
	pub opcode:  Opcode,
	pub unique:  u64,
	pub nodeid:  u64,
	pub uid:     u32,
	pub gid:     u32,
	pub pid:     u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_out_header {
	pub len:    u32,
	pub error:  i32,
	pub unique: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_dirent {
	pub ino:     u64,
	pub off:     u64,
	pub namelen: u32,
	pub r#type:  u32,
	pub name:    [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_direntplus {
	pub entry_out: fuse_entry_out,
	pub dirent:    fuse_dirent,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_inval_inode_out {
	pub ino: u64,
	pub off: i64,
	pub len: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_inval_entry_out {
	pub parent:  u64,
	pub namelen: u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_delete_out {
	pub parent:  u64,
	pub child:   u64,
	pub namelen: u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_store_out {
	pub nodeid:  u64,
	pub offset:  u64,
	pub size:    u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_retrieve_out {
	pub notify_unique: u64,
	pub nodeid:        u64,
	pub offset:        u64,
	pub size:          u32,
	pub padding:       u32,
}

/* Matches the size of fuse_write_in */
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_notify_retrieve_in {
	pub dummy1: u64,
	pub offset: u64,
	pub size:   u32,
	pub dummy2: u32,
	pub dummy3: u64,
	pub dummy4: u64,
}

/* Device ioctls: */
#[cfg(target_os = "linux")]
pub const FUSE_DEV_IOC_CLONE: u32 = _IOR!(229, 0, uint32_t);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_lseek_in {
	pub fh:      u64,
	pub offset:  u64,
	pub whence:  u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct fuse_lseek_out {
	pub offset: u64,
}
