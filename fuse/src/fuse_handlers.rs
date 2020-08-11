// Copyright (C) 2001-2007 Miklos Szeredi <miklos@szeredi.hu>
//
// This file is derived from `include/fuse_lowlevel.h` in the libfuse
// project. It may be used under the terms of the GNU Lesser General Public
// License, version 2.1 ("LGPL").
//
// The full terms of the LGPL can be found in the `licenses/lgpl-2.1.txt` file.
//
// SPDX-License-Identifier: LGPL-2.1-only

use std::io;

use crate::internal::errors;
use crate::protocol;
use crate::server;

/// User-provided handlers for FUSE operations.
///
/// Most FUSE handlers (with the exception of [`FuseHandlers::fuse_init`]) are
/// asynchronous. These handlers receive a [`ServerContext`] containing
/// information about the request itself, along with a [`ServerResponseWriter`]
/// that must be used to send the response.
///
/// The default implementation for all async handlers is to respond with
/// error code `ENOSYS`.
///
/// [`FuseHandlers::fuse_init`]: #method.fuse_init
/// [`ServerContext`]: struct.ServerContext.html
/// [`ServerResponseWriter`]: struct.ServerResponseWriter.html
pub trait FuseHandlers {
	/// Initialize the FUSE connection parameters.
	///
	/// Most servers do not need to override this method.
	///
	/// The default implementation returns a response created by the helper
	/// function [`FuseInitResponse::for_request`], which is also a good starting
	/// point for custom implementations.
	///
	/// [`FuseInitResponse::for_request`]: protocol/struct.FuseInitResponse.html#method.for_request
	fn fuse_init(
		&mut self,
		request: &protocol::FuseInitRequest,
	) -> protocol::FuseInitResponse {
		protocol::FuseInitResponse::for_request_impl(request)
	}

	/// **\[UNSTABLE\]** Check file access permissions
	///
	/// This will be called for the [`access(2)`] and [`chdir(2)`] system
	/// calls.  If the `default_permissions` mount option is given,
	/// this method is not called.
	///
	/// This method is not called under Linux kernel versions 2.4.x
	///
	/// If this request is answered with an error code of `ENOSYS`, this is
	/// treated as a permanent success, i.e. this and all future `access()`
	/// requests will succeed without being send to the filesystem process.
	///
	/// [`access(2)`]: http://pubs.opengroup.org/onlinepubs/9699919799/functions/access.html
	/// [`chdir(2)`]: http://pubs.opengroup.org/onlinepubs/9699919799/functions/chdir.html
	#[cfg(any(doc, feature = "unstable_fuse_access"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_access")))]
	fn access(
		&self,
		ctx: server::ServerContext,
		request: &protocol::AccessRequest,
		respond: impl for<'a> server::RespondOnce<protocol::AccessResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}


	/// **\[UNSTABLE\]** Map block index within file to block index within device
	///
	/// Note: This makes sense only for block device backed filesystems
	/// mounted with the `blkdev` option
	///
	/// If this request is answered with an error code of `ENOSYS`, this is
	/// treated as a permanent failure, i.e. all future `bmap()` requests will
	/// fail with the same error code without being send to the filesystem
	/// process.
	#[cfg(any(doc, feature = "unstable_fuse_bmap"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_bmap")))]
	fn bmap(
		&self,
		ctx: server::ServerContext,
		request: &protocol::BmapRequest,
		respond: impl for<'a> server::RespondOnce<protocol::BmapResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Create and open a file
	///
	/// If the file does not exist, first create it with the specified
	/// mode, and then open it.
	///
	/// See the description of [`FuseHandlers::open`] for more
	/// information.
	///
	/// If this method is not implemented or under Linux kernel
	/// versions earlier than 2.6.15, the [`FuseHandlers::mknod`] and
	/// [`FuseHandlers::open`] handlers will be called instead.
	///
	/// If this request is answered with an error code of `ENOSYS`, the handler
	/// is treated as not implemented (i.e., for this and future requests the
	/// [`FuseHandlers::mknod`] and [`FuseHandlers::open`] handlers will be
	/// called instead).
	///
	/// [`FuseHandlers::mknod`]: #method.mknod
	/// [`FuseHandlers::open`]: #method.open
	#[cfg(any(doc, feature = "unstable_fuse_create"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_create")))]
	fn create(
		&self,
		ctx: server::ServerContext,
		request: &protocol::CreateRequest,
		respond: impl for<'a> server::RespondOnce<protocol::CreateResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	// **\[UNSTABLE\]** Allocate requested space. If this function returns success then
	// subsequent writes to the specified range shall not fail due to the lack
	// of free space on the file system storage media.
	//
	// If this request is answered with an error code of `ENOSYS`, this is
	// treated as a permanent failure with error code `EOPNOTSUPP`, i.e. all
	// future `fallocate()` requests will fail with `EOPNOTSUPP` without being
	// send to the filesystem process.
	#[cfg(any(doc, feature = "unstable_fuse_fallocate"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_fallocate")))]
	fn fallocate(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FallocateRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FallocateResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Flush method
	///
	/// This is called on each `close()` of the opened file.
	///
	/// Since file descriptors can be duplicated (`dup`, `dup2`, `fork`), for
	/// one open call there may be many flush calls.
	///
	/// Filesystems shouldn't assume that flush will always be called
	/// after some writes, or that if will be called at all.
	///
	/// fi->fh will contain the value set by the open method, or will
	/// be undefined if the open method didn't set any value.
	///
	/// NOTE: the name of the method is misleading, since (unlike
	/// fsync) the filesystem is not forced to flush pending writes.
	/// One reason to flush data is if the filesystem wants to return
	/// write errors during close.  However, such use is non-portable
	/// because POSIX does not require [close] to wait for delayed I/O to
	/// complete.
	///
	/// If the filesystem supports file locking operations (setlk,
	/// getlk) it should remove all locks belonging to 'fi->owner'.
	///
	/// If this request is answered with an error code of ENOSYS,
	/// this is treated as success and future calls to flush() will
	/// succeed automatically without being send to the filesystem
	/// process.
	///
	/// [close]: http://pubs.opengroup.org/onlinepubs/9699919799/functions/close.html
	#[cfg(any(doc, feature = "unstable_fuse_flush"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_flush")))]
	fn flush(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FlushRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FlushResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Forget about an inode
	///
	/// This function is called when the kernel removes an inode
	/// from its internal caches.
	///
	/// The inode's lookup count increases by one for every call to
	/// fuse_reply_entry and fuse_reply_create. The nlookup parameter
	/// indicates by how much the lookup count should be decreased.
	///
	/// Inodes with a non-zero lookup count may receive request from
	/// the kernel even after calls to unlink, rmdir or (when
	/// overwriting an existing file) rename. Filesystems must handle
	/// such requests properly and it is recommended to defer removal
	/// of the inode until the lookup count reaches zero. Calls to
	/// unlink, rmdir or rename will be followed closely by forget
	/// unless the file or directory is open, in which case the
	/// kernel issues forget only after the release or releasedir
	/// calls.
	///
	/// Note that if a file system will be exported over NFS the
	/// inodes lifetime must extend even beyond forget. See the
	/// generation field in struct fuse_entry_param above.
	///
	/// On unmount the lookup count for all inodes implicitly drops
	/// to zero. It is not guaranteed that the file system will
	/// receive corresponding forget messages for the affected
	/// inodes.
	#[cfg(any(doc, feature = "unstable_fuse_forget"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_forget")))]
	fn forget(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ForgetRequest,
	) {
		let _ = (ctx, request);
	}

	/// **\[UNSTABLE\]** Synchronize file contents
	///
	/// If the datasync parameter is non-zero, then only the user data
	/// should be flushed, not the meta data.
	///
	/// If this request is answered with an error code of ENOSYS,
	/// this is treated as success and future calls to fsync() will
	/// succeed automatically without being send to the filesystem
	/// process.
	#[cfg(any(doc, feature = "unstable_fuse_fsync"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_fsync")))]
	fn fsync(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FsyncResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Synchronize directory contents
	///
	/// If the datasync parameter is non-zero, then only the directory
	/// contents should be flushed, not the meta data.
	///
	/// fi->fh will contain the value set by the opendir method, or
	/// will be undefined if the opendir method didn't set any value.
	///
	/// If this request is answered with an error code of ENOSYS,
	/// this is treated as success and future calls to fsyncdir() will
	/// succeed automatically without being send to the filesystem
	/// process.
	#[cfg(any(doc, feature = "unstable_fuse_fsyncdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_fsyncdir")))]
	fn fsyncdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::FsyncdirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::FsyncdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Get file attributes.
	///
	/// If writeback caching is enabled, the kernel may have a
	/// better idea of a file's length than the FUSE file system
	/// (eg if there has been a write that extended the file size,
	/// but that has not yet been passed to the filesystem.
	///
	/// In this case, the [`NodeAttr::size`] value provided by the file system
	/// will be ignored.
	///
	/// [`NodeAttr::size`]: protocol/struct.NodeAttr.html#method.size
	#[cfg(any(doc, feature = "unstable_fuse_getattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_getattr")))]
	fn getattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::GetattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Test for a POSIX file lock
	#[cfg(any(doc, feature = "unstable_fuse_getlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_getlk")))]
	fn getlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetlkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::GetlkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Get an extended attribute
	///
	/// If size is zero, the size of the value should be sent with
	/// fuse_reply_xattr.
	///
	/// If the size is non-zero, and the value fits in the buffer, the
	/// value should be sent with fuse_reply_buf.
	///
	/// If the size is too small for the value, the ERANGE error should
	/// be sent.
	///
	/// If this request is answered with an error code of ENOSYS, this is
	/// treated as a permanent failure with error code EOPNOTSUPP, i.e. all
	/// future getxattr() requests will fail with EOPNOTSUPP without being
	/// send to the filesystem process.
	#[cfg(any(doc, feature = "unstable_fuse_getxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_getxattr")))]
	fn getxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::GetxattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::GetxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Ioctl
	///
	/// Note: For unrestricted ioctls (not allowed for FUSE
	/// servers), data in and out areas can be discovered by giving
	/// iovs and setting FUSE_IOCTL_RETRY in *flags*.  For
	/// restricted ioctls, kernel prepares in/out data area
	/// according to the information encoded in cmd.
	///
	/// Valid replies:
	///   fuse_reply_ioctl_retry
	///   fuse_reply_ioctl
	///   fuse_reply_ioctl_iov
	///   fuse_reply_err
	///
	/// @param req request handle
	/// @param ino the inode number
	/// @param cmd ioctl command
	/// @param arg ioctl argument
	/// @param fi file information
	/// @param flags for FUSE_IOCTL_* flags
	/// @param in_buf data fetched from the caller
	/// @param in_bufsz number of fetched bytes
	/// @param out_bufsz maximum size of output data
	///
	/// Note : the unsigned long request submitted by the application
	/// is truncated to 32 bits.
	#[cfg(any(doc, feature = "unstable_fuse_ioctl"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_ioctl")))]
	fn ioctl(
		&self,
		ctx: server::ServerContext,
		request: &protocol::IoctlRequest,
		respond: impl for<'a> server::RespondOnce<protocol::IoctlResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Create a hard link
	#[cfg(any(doc, feature = "unstable_fuse_link"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_link")))]
	fn link(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::LinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** List extended attribute names
	///
	/// If size is zero, the total size of the attribute list should be
	/// sent with fuse_reply_xattr.
	///
	/// If the size is non-zero, and the null character separated
	/// attribute list fits in the buffer, the list should be sent with
	/// fuse_reply_buf.
	///
	/// If the size is too small for the list, the ERANGE error should
	/// be sent.
	///
	/// If this request is answered with an error code of ENOSYS, this is
	/// treated as a permanent failure with error code EOPNOTSUPP, i.e. all
	/// future listxattr() requests will fail with EOPNOTSUPP without being
	/// send to the filesystem process.
	#[cfg(any(doc, feature = "unstable_fuse_listxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_listxattr")))]
	fn listxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ListxattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ListxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Look up a directory entry by name and get its attributes.
	#[cfg(any(doc, feature = "unstable_fuse_lookup"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_lookup")))]
	fn lookup(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LookupRequest,
		respond: impl for<'a> server::RespondOnce<protocol::LookupResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Find next data or hole after the specified offset
	///
	/// If this request is answered with an error code of ENOSYS, this is
	/// treated as a permanent failure, i.e. all future lseek() requests will
	/// fail with the same error code without being send to the filesystem
	/// process.
	#[cfg(any(doc, feature = "unstable_fuse_lseek"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_lseek")))]
	fn lseek(
		&self,
		ctx: server::ServerContext,
		request: &protocol::LseekRequest,
		respond: impl for<'a> server::RespondOnce<protocol::LseekResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Create a directory
	#[cfg(any(doc, feature = "unstable_fuse_mkdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_mkdir")))]
	fn mkdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MkdirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::MkdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Create file node
	///
	/// Create a regular file, character device, block device, fifo or
	/// socket node.
	#[cfg(any(doc, feature = "unstable_fuse_mknod"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_mknod")))]
	fn mknod(
		&self,
		ctx: server::ServerContext,
		request: &protocol::MknodRequest,
		respond: impl for<'a> server::RespondOnce<protocol::MknodResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// Open a file
	///
	/// Platform-specific flags passed to [`open(2)`] are available in
	/// [`OpenRequest::flags`]. The following rules apply:
	///
	///  - Creation (`O_CREAT`, `O_EXCL`, `O_NOCTTY`) flags will be filtered out
	///    / handled by the kernel.
	///
	///  - Access modes (`O_RDONLY`, `O_WRONLY`, `O_RDWR`) should be used by the
	///    filesystem to check if the operation is permitted. If the
	///    `-o default_permissions` mount option is given, this check is already
	///    done by the kernel before calling `open` and may thus be omitted by
	///    the filesystem.
	///
	///  - When writeback caching is enabled, the kernel may send read requests
	///    even for files opened with `O_WRONLY`. The filesystem should be
	///    prepared to handle this.
	///
	///  - When writeback caching is disabled, the filesystem is expected to
	///    properly handle the `O_APPEND` flag and ensure that each write is
	///    appending to the end of the file.
	///
	///  - When writeback caching is enabled, the kernel will handle `O_APPEND`.
	///    However, unless all changes to the file come through the kernel this
	///    will not work reliably. The filesystem should thus either ignore the
	///    `O_APPEND` flag (and let the kernel handle it), or return an error
	///    (indicating that reliable `O_APPEND` is not available).
	///
	/// Filesystem may store an arbitrary file handle (pointer, index, etc) with
	/// [`OpenResponse::set_handle`], and use this in other all other file
	/// operations (`read`, `write`, `flush`, `release`, `fsync`).
	///
	/// Filesystem may also implement stateless file I/O and not store anything
	/// with [`OpenResponse::set_handle`].
	///
	/// There are also some flags (`direct_io`, `keep_cache`) which the
	/// filesystem may set in the response, to change the way the file is opened.
	/// See [`OpenFlags`] for more details.
	///
	/// If this request is answered with an error code of `ENOSYS` and
	/// `no_open_support` was set in [`FuseInitFlags`], this is treated as success
	/// and future calls to `open` and `release` will also succeed without being
	/// sent to the filesystem process.
	///
	/// [`FuseInitFlags`]: protocol/struct.FuseInitFlags.html
	/// [`open(2)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/open.html
	/// [`OpenFlags`]: protocol/struct.OpenFlags.html
	/// [`OpenRequest::flags`]: protocol/struct.OpenRequest.html#method.flags
	/// [`OpenResponse::set_handle`]: protocol/struct.OpenResponse.html#method.set_handle
	fn open(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpenRequest,
		respond: impl for<'a> server::RespondOnce<protocol::OpenResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Open a directory
	///
	/// Filesystem may store an arbitrary file handle (pointer, index,
	/// etc) in fi->fh, and use this in other all other directory
	/// stream operations (readdir, releasedir, fsyncdir).
	///
	/// If this request is answered with an error code of ENOSYS and
	/// FUSE_CAP_NO_OPENDIR_SUPPORT is set in `fuse_conn_info.capable`,
	/// this is treated as success and future calls to opendir and
	/// releasedir will also succeed without being sent to the filesystem
	/// process. In addition, the kernel will cache readdir results
	/// as if opendir returned FOPEN_KEEP_CACHE | FOPEN_CACHE_DIR.
	#[cfg(any(doc, feature = "unstable_fuse_opendir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_opendir")))]
	fn opendir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::OpendirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::OpendirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// Read data
	///
	/// Read should send exactly the number of bytes requested except on EOF or
	/// error, otherwise the rest of the data will be substituted with zeroes. An
	/// exception to this is when the file has been opened in `direct_io` mode,
	/// in which case the return value of the read system call will reflect the
	/// response from this operation.
	///
	/// [`ReadRequest::handle`] will return the value set by the open method,
	/// or will return 0 if the open method didn't set any value.
	///
	/// [`ReadRequest::handle`]: protocol/struct.ReadRequest.html#method.handle
	fn read(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReadResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Read directory
	///
	/// Send a buffer filled using fuse_add_direntry(), with size not
	/// exceeding the requested size.  Send an empty buffer on end of
	/// stream.
	///
	/// fi->fh will contain the value set by the opendir method, or
	/// will be undefined if the opendir method didn't set any value.
	///
	/// Returning a directory entry from readdir() does not affect
	/// its lookup count.
	///
	/// If off_t is non-zero, then it will correspond to one of the off_t
	/// values that was previously returned by readdir() for the same
	/// directory handle. In this case, readdir() should skip over entries
	/// coming before the position defined by the off_t value. If entries
	/// are added or removed while the directory handle is open, they filesystem
	/// may still include the entries that have been removed, and may not
	/// report the entries that have been created. However, addition or
	/// removal of entries must never cause readdir() to skip over unrelated
	/// entries or to report them more than once. This means
	/// that off_t can not be a simple index that enumerates the entries
	/// that have been returned but must contain sufficient information to
	/// uniquely determine the next directory entry to return even when the
	/// set of entries is changing.
	///
	/// The function does not have to report the '.' and '..'
	/// entries, but is allowed to do so. Note that, if readdir does
	/// not return '.' or '..', they will not be implicitly returned,
	/// and this behavior is observable by the caller.
	#[cfg(any(doc, feature = "unstable_fuse_readdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_readdir")))]
	fn readdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReaddirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReaddirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Read symbolic link
	#[cfg(any(doc, feature = "unstable_fuse_readlink"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_readlink")))]
	fn readlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReadlinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReadlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Release an open file
	///
	/// Release is called when there are no more references to an open
	/// file: all file descriptors are closed and all memory mappings
	/// are unmapped.
	///
	/// For every open call there will be exactly one release call (unless
	/// the filesystem is force-unmounted).
	///
	/// The filesystem may reply with an error, but error values are
	/// not returned to close() or munmap() which triggered the
	/// release.
	///
	/// fi->fh will contain the value set by the open method, or will
	/// be undefined if the open method didn't set any value.
	/// fi->flags will contain the same flags as for open.
	#[cfg(any(doc, feature = "unstable_fuse_release"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_release")))]
	fn release(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleaseRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReleaseResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Release an open directory
	///
	/// For every opendir call there will be exactly one releasedir
	/// call (unless the filesystem is force-unmounted).
	///
	/// fi->fh will contain the value set by the opendir method, or
	/// will be undefined if the opendir method didn't set any value.
	#[cfg(any(doc, feature = "unstable_fuse_releasedir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_releasedir")))]
	fn releasedir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::ReleasedirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::ReleasedirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Remove an extended attribute
	///
	/// If this request is answered with an error code of ENOSYS, this is
	/// treated as a permanent failure with error code EOPNOTSUPP, i.e. all
	/// future removexattr() requests will fail with EOPNOTSUPP without being
	/// send to the filesystem process.
	#[cfg(any(doc, feature = "unstable_fuse_removexattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_removexattr")))]
	fn removexattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RemovexattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::RemovexattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Rename a file
	///
	/// If the target exists it should be atomically replaced. If
	/// the target's inode's lookup count is non-zero, the file
	/// system is expected to postpone any removal of the inode
	/// until the lookup count reaches zero (see description of the
	/// forget function).
	///
	/// If this request is answered with an error code of ENOSYS, this is
	/// treated as a permanent failure with error code EINVAL, i.e. all
	/// future bmap requests will fail with EINVAL without being
	/// send to the filesystem process.
	///
	/// *flags* may be `RENAME_EXCHANGE` or `RENAME_NOREPLACE`. If
	/// RENAME_NOREPLACE is specified, the filesystem must not
	/// overwrite *newname* if it exists and return an error
	/// instead. If `RENAME_EXCHANGE` is specified, the filesystem
	/// must atomically exchange the two files, i.e. both must
	/// exist and neither may be deleted.
	#[cfg(any(doc, feature = "unstable_fuse_rename"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_rename")))]
	fn rename(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RenameRequest,
		respond: impl for<'a> server::RespondOnce<protocol::RenameResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Remove a directory
	///
	/// If the directory's inode's lookup count is non-zero, the
	/// file system is expected to postpone any removal of the
	/// inode until the lookup count reaches zero (see description
	/// of the forget function).
	#[cfg(any(doc, feature = "unstable_fuse_rmdir"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_rmdir")))]
	fn rmdir(
		&self,
		ctx: server::ServerContext,
		request: &protocol::RmdirRequest,
		respond: impl for<'a> server::RespondOnce<protocol::RmdirResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Set file attributes
	///
	/// In the 'attr' argument only members indicated by the 'to_set'
	/// bitmask contain valid values.  Other members contain undefined
	/// values.
	///
	/// Unless FUSE_CAP_HANDLE_KILLPRIV is disabled, this method is
	/// expected to reset the setuid and setgid bits if the file
	/// size or owner is being changed.
	///
	/// If the setattr was invoked from the ftruncate() system call
	/// under Linux kernel versions 2.6.15 or later, the fi->fh will
	/// contain the value set by the open method or will be undefined
	/// if the open method didn't set any value.  Otherwise (not
	/// ftruncate call, or kernel version earlier than 2.6.15) the fi
	/// parameter will be NULL.
	#[cfg(any(doc, feature = "unstable_fuse_setattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_setattr")))]
	fn setattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SetattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Acquire, modify or release a POSIX file lock
	///
	/// For POSIX threads (NPTL) there's a 1-1 relation between pid and
	/// owner, but otherwise this is not always the case.  For checking
	/// lock ownership, 'fi->owner' must be used.  The l_pid field in
	/// 'struct flock' should only be used to fill in this field in
	/// getlk().
	///
	/// Note: if the locking methods are not implemented, the kernel
	/// will still allow file locking to work locally.  Hence these are
	/// only interesting for network filesystems and similar.
	#[cfg(any(doc, feature = "unstable_fuse_setlk"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_setlk")))]
	fn setlk(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetlkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SetlkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Set an extended attribute
	///
	/// If this request is answered with an error code of ENOSYS, this is
	/// treated as a permanent failure with error code EOPNOTSUPP, i.e. all
	/// future setxattr() requests will fail with EOPNOTSUPP without being
	/// send to the filesystem process.
	#[cfg(any(doc, feature = "unstable_fuse_setxattr"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_setxattr")))]
	fn setxattr(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SetxattrRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SetxattrResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Get file system statistics
	#[cfg(any(doc, feature = "unstable_fuse_statfs"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_statfs")))]
	fn statfs(
		&self,
		ctx: server::ServerContext,
		request: &protocol::StatfsRequest,
		respond: impl for<'a> server::RespondOnce<protocol::StatfsResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Create a symbolic link
	#[cfg(any(doc, feature = "unstable_fuse_symlink"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_symlink")))]
	fn symlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::SymlinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::SymlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Remove a file
	///
	/// If the file's inode's lookup count is non-zero, the file
	/// system is expected to postpone any removal of the inode
	/// until the lookup count reaches zero (see description of the
	/// forget function).
	#[cfg(any(doc, feature = "unstable_fuse_unlink"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_unlink")))]
	fn unlink(
		&self,
		ctx: server::ServerContext,
		request: &protocol::UnlinkRequest,
		respond: impl for<'a> server::RespondOnce<protocol::UnlinkResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}

	/// **\[UNSTABLE\]** Write data
	///
	/// Write should return exactly the number of bytes requested
	/// except on error.  An exception to this is when the file has
	/// been opened in 'direct_io' mode, in which case the return value
	/// of the write system call will reflect the return value of this
	/// operation.
	///
	/// Unless FUSE_CAP_HANDLE_KILLPRIV is disabled, this method is
	/// expected to reset the setuid and setgid bits.
	///
	/// fi->fh will contain the value set by the open method, or will
	/// be undefined if the open method didn't set any value.
	#[cfg(any(doc, feature = "unstable_fuse_write"))]
	#[cfg_attr(doc, doc(cfg(feature = "unstable_fuse_write")))]
	fn write(
		&self,
		ctx: server::ServerContext,
		request: &protocol::WriteRequest,
		respond: impl for<'a> server::RespondOnce<protocol::WriteResponse<'a>>,
	) {
		let _ = (ctx, request);
		respond.err(errors::ENOSYS);
	}
}
