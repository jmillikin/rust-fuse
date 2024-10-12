// Copyright 2021 John Millikin and the rust-fuse contributors.
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

//! CUSE and FUSE servers.

pub(crate) mod decode;

use core::cmp;
use core::fmt;
use core::marker::PhantomData;
use core::mem::size_of;
use core::num::NonZeroU64;
use core::ptr::NonNull;

use crate::{
	CuseDeviceName,
	CuseDeviceNumber,
};
use crate::io::{AlignedSlice, SendBuf};
use crate::kernel;
use crate::kernel::fuse_opcode;
use crate::operations::cuse_init::{
	CuseInitFlag,
	CuseInitFlags,
};
use crate::operations::fuse_init::{
	FuseInitFlag,
	FuseInitFlags,
};

pub use crate::operations::{
	access::AccessRequest,
	bmap::BmapRequest,
	copy_file_range::CopyFileRangeRequest,
	create::{CreateRequest, CreateResponse},
	cuse_init::{CuseInitRequest, CuseInitResponse},
	fallocate::FallocateRequest,
	flush::FlushRequest,
	forget::{ForgetRequest, ForgetRequestItem},
	fsync::FsyncRequest,
	fsyncdir::FsyncdirRequest,
	fuse_init::{FuseInitRequest, FuseInitResponse},
	getattr::GetattrRequest,
	getlk::GetlkRequest,
	getxattr::GetxattrRequest,
	interrupt::InterruptRequest,
	ioctl::{
		IoctlPtr,
		IoctlRequest,
		IoctlResponse,
		IoctlRetryBuf,
	},
	link::LinkRequest,
	listxattr::{ListxattrNamesWriter, ListxattrRequest},
	lookup::LookupRequest,
	lseek::LseekRequest,
	mkdir::MkdirRequest,
	mknod::MknodRequest,
	open::OpenRequest,
	opendir::OpendirRequest,
	poll::PollRequest,
	read::ReadRequest,
	readdir::{
		ReaddirEntry,
		ReaddirEntries,
		ReaddirEntriesWriter,
		ReaddirRequest,
	},
	readdirplus::{
		ReaddirplusEntry,
		ReaddirplusEntries,
		ReaddirplusEntriesWriter,
		ReaddirplusRequest,
	},
	readlink::ReadlinkRequest,
	release::ReleaseRequest,
	releasedir::ReleasedirRequest,
	removexattr::RemovexattrRequest,
	rename::RenameRequest,
	rmdir::RmdirRequest,
	setattr::SetattrRequest,
	setlk::SetlkRequest,
	setxattr::SetxattrRequest,
	statfs::StatfsRequest,
	symlink::SymlinkRequest,
	unlink::UnlinkRequest,
	write::WriteRequest,
};

/// Errors that may be encountered when receiving a request.
///
/// Sockets may use the variants of this enum to provide hints to server code
/// about the nature and severity of errors.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecvError<IoError> {
	/// The connection has been cleanly closed by the client.
	ConnectionClosed(IoError),

	/// The socket encountered an error not otherwise specified.
	Other(IoError),
}

/// Errors that may be encountered when sending a reply.
///
/// Sockets may use the variants of this enum to provide hints to server code
/// about the nature and severity of errors.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SendError<IoError> {
	/// The reply's original request has been forgotten by the client.
	///
	/// The server should treat this as a non-fatal error.
	NotFound(IoError),

	/// The reply's size would exceed the FUSE protocol's maximum limit
	/// of [`u32::MAX`] bytes.
	ReplyTooBig(u64),

	/// The socket encountered an error not otherwise specified.
	Other(IoError),
}

/// Trait for sockets that can receive requests and send replies.
pub trait Socket {
	/// Type of errors that may be returned from this socket's I/O methods.
	type Error;

	/// Receive a single serialised request from the client.
	///
	/// The buffer must be large enough to contain any request that might be
	/// received for the current session's negotiated maximum message size.
	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<Self::Error>>;

	/// Send a single serialised reply to the client.
	fn send(&self, buf: SendBuf) -> Result<(), SendError<Self::Error>>;
}

impl<S: Socket> Socket for &S {
	type Error = S::Error;

	fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError<S::Error>> {
		(*self).recv(buf)
	}

	fn send(&self, buf: SendBuf) -> Result<(), SendError<S::Error>> {
		(*self).send(buf)
	}
}

/// Marker trait for CUSE sockets.
pub trait CuseSocket: Socket {}

impl<S: CuseSocket> CuseSocket for &S {}

/// Marker trait for FUSE sockets.
pub trait FuseSocket: Socket {}

impl<S: FuseSocket> FuseSocket for &S {}

/// Errors that may be encountered by a CUSE or FUSE server.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServerError<IoError> {
	/// An invalid request was received from the client.
	RequestError(RequestError),

	/// The socket encountered an I/O error when receiving the next request.
	RecvError(RecvError<IoError>),

	/// The socket encountered an I/O error when sending a reply.
	SendError(SendError<IoError>),
}

impl<E> From<RequestError> for ServerError<E> {
	fn from(err: RequestError) -> Self {
		Self::RequestError(err)
	}
}

impl<E> From<RecvError<E>> for ServerError<E> {
	fn from(err: RecvError<E>) -> Self {
		Self::RecvError(err)
	}
}

impl<E> From<SendError<E>> for ServerError<E> {
	fn from(err: SendError<E>) -> Self {
		Self::SendError(err)
	}
}

#[allow(missing_docs)] // TODO
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutError {
	Todo,
}

#[allow(missing_docs)] // TODO
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CuseLayout {
	version_minor: u16,
}

impl CuseLayout {
	#[allow(missing_docs)] // TODO
	pub const fn new(
		init_out: &kernel::cuse_init_out,
	) -> Result<CuseLayout, LayoutError> {
		if init_out.major != kernel::FUSE_KERNEL_VERSION {
			return Err(LayoutError::Todo);
		}
		if init_out.minor > kernel::FUSE_KERNEL_MINOR_VERSION {
			return Err(LayoutError::Todo);
		}
		Ok(Self {
			version_minor: init_out.minor as u16,
		})
	}

	#[inline]
	pub(crate) fn version_minor(self) -> u32 {
		u32::from(self.version_minor)
	}
}

#[allow(missing_docs)] // TODO
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FuseLayout {
	pub(crate) version_minor: u16,
	pub(crate) features: u16,
}

const FEATURE_SETXATTR_EXT: u16 = 1 << 0;

impl FuseLayout {
	#[allow(missing_docs)] // TODO
	pub const fn new(
		init_out: &kernel::fuse_init_out,
	) -> Result<FuseLayout, LayoutError> {
		if init_out.major != kernel::FUSE_KERNEL_VERSION {
			return Err(LayoutError::Todo);
		}
		if init_out.minor > kernel::FUSE_KERNEL_MINOR_VERSION {
			return Err(LayoutError::Todo);
		}
		Ok(Self::new2(init_out))
	}

	pub(crate) const fn new2(init_out: &kernel::fuse_init_out) -> FuseLayout {
		let mut features = 0;
		if init_out.flags & kernel::FUSE_SETXATTR_EXT != 0 {
			features |= FEATURE_SETXATTR_EXT;
		}
		Self {
			version_minor: init_out.minor as u16,
			features,
		}
	}

	#[inline]
	pub(crate) fn version_minor(self) -> u32 {
		u32::from(self.version_minor)
	}

	#[must_use]
	pub(crate) fn have_setxattr_ext(self) -> bool {
		self.features & FEATURE_SETXATTR_EXT != 0
	}
}

/// Errors describing why a request is invalid.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestError {
	/// The request contains an invalid [`Lock`].
	///
	/// [`Lock`]: crate::Lock
	LockError(crate::LockError),

	/// The request is missing one or mode node IDs.
	///
	/// For most requests this will mean that the [`RequestHeader::node_id`]
	/// is `None`, but some request types have additional required node IDs
	/// in the request body.
	///
	/// [`RequestHeader::node_id`]: crate::RequestHeader::node_id
	MissingNodeId,

	/// The request is a `FUSE_INTERRUPT` with a missing request ID.
	MissingRequestId,

	/// The request contains an invalid [`crate::NodeName`].
	NodeNameError(crate::NodeNameError),

	/// The request contains a timestamp with too many nanoseconds.
	TimestampOverflow,

	/// The request contains an invalid [`crate::XattrName`].
	///
	/// [`crate::XattrName`]: crate::XattrName
	XattrNameError(crate::XattrNameError),

	/// The request contains an invalid [`crate::XattrValue`].
	///
	/// [`crate::XattrValue`]: crate::XattrValue
	XattrValueError(crate::XattrValueError),

	// Errors indicating a programming error in the client.

	/// The request header's request ID is zero.
	InvalidRequestId,

	/// The request buffer contains an incomplete request.
	UnexpectedEof,

	// Errors indicating a programming error in the server.

	/// Attempted to decode a request as the wrong type.
	///
	/// This error indicates a programming error in the server.
	OpcodeMismatch,
}

impl From<crate::LockError> for RequestError {
	fn from(err: crate::LockError) -> RequestError {
		RequestError::LockError(err)
	}
}

impl From<crate::NodeNameError> for RequestError {
	fn from(err: crate::NodeNameError) -> RequestError {
		RequestError::NodeNameError(err)
	}
}

impl From<crate::XattrNameError> for RequestError {
	fn from(err: crate::XattrNameError) -> RequestError {
		RequestError::XattrNameError(err)
	}
}

impl From<crate::XattrValueError> for RequestError {
	fn from(err: crate::XattrValueError) -> RequestError {
		RequestError::XattrValueError(err)
	}
}

#[derive(Clone, Copy)]
pub(crate) struct Request<'a> {
	pub(crate) ptr: NonNull<u8>,
	_ptr: PhantomData<&'a kernel::fuse_in_header>,
}

impl<'a> Request<'a> {
	fn new(buf: AlignedSlice<'a>) -> Result<Request<'a>, RequestError> {
		let buf = buf.get();
		if buf.len() < size_of::<kernel::fuse_in_header>() {
			return Err(RequestError::UnexpectedEof);
		}

		let header_ptr = buf.as_ptr().cast::<kernel::fuse_in_header>();
		let header = unsafe { &*header_ptr };

		if header.unique == 0 {
			return Err(RequestError::InvalidRequestId);
		}

		let buf_len: u32;
		if size_of::<usize>() > size_of::<u32>() {
			if buf.len() > u32::MAX as usize {
				buf_len = u32::MAX;
			} else {
				buf_len = buf.len() as u32;
			}
		} else {
			buf_len = buf.len() as u32;
		}
		if buf_len < header.len {
			return Err(RequestError::UnexpectedEof);
		}

		Ok(Self {
			ptr: unsafe { NonNull::new_unchecked(buf.as_ptr().cast_mut()) },
			_ptr: PhantomData,
		})
	}

	#[must_use]
	fn as_bytes(self) -> &'a [u8] {
		let len = self.raw_header().len as usize;
		let buf_ptr = NonNull::slice_from_raw_parts(self.ptr, len);
		unsafe { buf_ptr.as_ref() }
	}

	#[must_use]
	pub(crate) fn header(self) -> &'a crate::RequestHeader {
		unsafe { self.ptr.cast().as_ref() }
	}

	#[must_use]
	fn raw_header(self) -> &'a kernel::fuse_in_header {
		unsafe { self.ptr.cast().as_ref() }
	}

	#[must_use]
	fn body(self) -> &'a [u8] {
		&self.as_bytes()[size_of::<kernel::fuse_in_header>()..]
	}

	#[must_use]
	pub(crate) fn decoder(self) -> decode::RequestDecoder<'a> {
		let buf = self.as_bytes();
		unsafe { decode::RequestDecoder::new_unchecked(buf) }
	}
}

#[allow(missing_docs)] // TODO
#[derive(Clone, Copy)]
pub struct CuseRequest<'a> {
	pub(crate) inner: Request<'a>,
	pub(crate) layout: CuseLayout,
}

impl<'a> CuseRequest<'a> {
	#[allow(missing_docs)] // TODO
	pub fn new(
		buf: crate::io::AlignedSlice<'a>,
		layout: CuseLayout,
	) -> Result<CuseRequest<'a>, RequestError> {
		let inner = Request::new(buf)?;
		Ok(CuseRequest { inner, layout })
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn as_bytes(self) -> &'a [u8] {
		self.inner.as_bytes()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn header(self) -> &'a crate::RequestHeader {
		self.inner.header()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn id(self) -> core::num::NonZeroU64 {
		self.header().request_id()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn body(self) -> &'a [u8] {
		self.inner.body()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub(crate) fn decoder(self) -> decode::RequestDecoder<'a> {
		self.inner.decoder()
	}
}

impl fmt::Debug for CuseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("CuseRequest")
			.field("header", &self.header())
			.field("body", &self.body())
			.finish()
	}
}

#[allow(missing_docs)] // TODO
#[derive(Clone, Copy)]
pub struct FuseRequest<'a> {
	pub(crate) inner: Request<'a>,
	pub(crate) layout: FuseLayout,
}

impl<'a> FuseRequest<'a> {
	#[allow(missing_docs)] // TODO
	pub fn new(
		buf: AlignedSlice<'a>,
		layout: FuseLayout,
	) -> Result<FuseRequest<'a>, RequestError> {
		let inner = Request::new(buf)?;
		Ok(FuseRequest { inner, layout })
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn as_bytes(self) -> &'a [u8] {
		self.inner.as_bytes()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn header(self) -> &'a crate::RequestHeader {
		self.inner.header()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn id(self) -> core::num::NonZeroU64 {
		self.header().request_id()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub fn body(self) -> &'a [u8] {
		self.inner.body()
	}

	#[allow(missing_docs)] // TODO
	#[must_use]
	pub(crate) fn decoder(self) -> decode::RequestDecoder<'a> {
		self.inner.decoder()
	}
}

impl fmt::Debug for FuseRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("FuseRequest")
			.field("header", &self.header())
			.field("body", &self.body())
			.finish()
	}
}

#[allow(missing_docs)] // TODO
pub trait CuseReply {
	#[allow(missing_docs)] // TODO
	fn send_to<S: CuseSocket>(
		&self,
		reply_sender: CuseReplySender<'_, S>,
	) -> Result<(), SendError<S::Error>>;
}

#[allow(missing_docs)] // TODO
pub trait FuseReply {
	#[allow(missing_docs)] // TODO
	fn send_to<S: FuseSocket>(
		&self,
		reply_sender: FuseReplySender<'_, S>,
	) -> Result<(), SendError<S::Error>>;
}

pub(crate) struct ReplySender<'a, S> {
	pub(crate) socket: &'a S,
	pub(crate) request_id: u64,
}

impl<'a, S: Socket> ReplySender<'a, S> {
	fn send_0(self, error: i32) -> Result<(), SendError<S::Error>> {
		let header = new!(kernel::fuse_out_header {
			len: core::mem::size_of::<kernel::fuse_out_header>() as u32,
			error: error,
			unique: self.request_id,
		});
		self.socket.send(SendBuf::new_1(
			header.len as usize,
			header.as_bytes(),
		))
	}

	pub(crate) fn send_1(
		self,
		bytes_1: &[u8],
	) -> Result<(), SendError<S::Error>> {
		let mut len = size_of::<kernel::fuse_out_header>() as u64;
		len = len.saturating_add(bytes_1.len() as u64);
		if len > u64::from(u32::MAX) {
			return Err(SendError::ReplyTooBig(len));
		}
		let header = new!(kernel::fuse_out_header {
			len: len as u32,
			unique: self.request_id,
		});
		self.socket.send(SendBuf::new_2(
			len as usize,
			header.as_bytes(),
			bytes_1,
		))
	}

	pub(crate) fn send_2(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
	) -> Result<(), SendError<S::Error>> {
		let mut len = size_of::<kernel::fuse_out_header>() as u64;
		len = len.saturating_add(bytes_1.len() as u64);
		len = len.saturating_add(bytes_2.len() as u64);
		if len > u64::from(u32::MAX) {
			return Err(SendError::ReplyTooBig(len));
		}
		let header = new!(kernel::fuse_out_header {
			len: len as u32,
			unique: self.request_id,
		});
		self.socket.send(SendBuf::new_3(
			len as usize,
			header.as_bytes(),
			bytes_1,
			bytes_2,
		))
	}

	pub(crate) fn send_3(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
		bytes_3: &[u8],
	) -> Result<(), SendError<S::Error>> {
		let mut len = size_of::<kernel::fuse_out_header>() as u64;
		len = len.saturating_add(bytes_1.len() as u64);
		len = len.saturating_add(bytes_2.len() as u64);
		len = len.saturating_add(bytes_3.len() as u64);
		if len > u64::from(u32::MAX) {
			return Err(SendError::ReplyTooBig(len));
		}
		let header = new!(kernel::fuse_out_header {
			len: len as u32,
			unique: self.request_id,
		});
		self.socket.send(SendBuf::new_4(
			len as usize,
			header.as_bytes(),
			bytes_1,
			bytes_2,
			bytes_3,
		))
	}

	pub(crate) fn send_4(
		self,
		bytes_1: &[u8],
		bytes_2: &[u8],
		bytes_3: &[u8],
		bytes_4: &[u8],
	) -> Result<(), SendError<S::Error>> {
		let mut len = size_of::<kernel::fuse_out_header>() as u64;
		len = len.saturating_add(bytes_1.len() as u64);
		len = len.saturating_add(bytes_2.len() as u64);
		len = len.saturating_add(bytes_3.len() as u64);
		len = len.saturating_add(bytes_4.len() as u64);
		if len > u64::from(u32::MAX) {
			return Err(SendError::ReplyTooBig(len));
		}
		let header = new!(kernel::fuse_out_header {
			len: len as u32,
			unique: self.request_id,
		});
		self.socket.send(SendBuf::new_5(
			len as usize,
			header.as_bytes(),
			bytes_1,
			bytes_2,
			bytes_3,
			bytes_4,
		))
	}
}

#[allow(missing_docs)] // TODO
#[must_use]
pub struct CuseReplySender<'a, S> {
	pub(crate) inner: ReplySender<'a, S>,
}

impl<'a, S: CuseSocket> CuseReplySender<'a, S> {
	#[allow(missing_docs)] // TODO
	pub fn new(
		socket: &'a S,
		layout: CuseLayout,
		request_id: NonZeroU64,
	) -> CuseReplySender<'a, S> {
		_ = layout;
		CuseReplySender {
			inner: ReplySender {
				socket,
				request_id: request_id.get(),
			},
		}
	}

	/// Send a successful reply to the CUSE client.
	pub fn ok(self, reply: &impl CuseReply) -> Result<(), SendError<S::Error>> {
		reply.send_to(self)
	}

	/// Send a successful reply (containing arbitrary data) to the CUSE client.
	pub fn ok_buf(self, buf: &[u8]) -> Result<(), SendError<S::Error>> {
		self.inner.send_1(buf)
	}

	/// Send a successful empty reply to the CUSE client.
	pub fn ok_empty(self) -> Result<(), SendError<S::Error>> {
		self.inner.send_0(0)
	}

	/// Send an error reply to the CUSE client.
	pub fn err(
		self,
		error: impl Into<crate::Error>,
	) -> Result<(), SendError<S::Error>> {
		self.inner.send_0(error.into().0.get())
	}
}

#[allow(missing_docs)] // TODO
#[must_use]
pub struct FuseReplySender<'a, S> {
	pub(crate) inner: ReplySender<'a, S>,
	pub(crate) layout: FuseLayout,
}

impl<'a, S: FuseSocket> FuseReplySender<'a, S> {
	#[allow(missing_docs)] // TODO
	pub fn new(
		socket: &'a S,
		layout: FuseLayout,
		request_id: NonZeroU64,
	) -> FuseReplySender<'a, S> {
		FuseReplySender {
			inner: ReplySender {
				socket,
				request_id: request_id.get(),
			},
			layout,
		}
	}

	/// Send a successful reply to the FUSE client.
	pub fn ok(self, reply: &impl FuseReply) -> Result<(), SendError<S::Error>> {
		reply.send_to(self)
	}

	/// Send a successful reply (containing arbitrary data) to the FUSE client.
	pub fn ok_buf(self, buf: &[u8]) -> Result<(), SendError<S::Error>> {
		self.inner.send_1(buf)
	}

	/// Send a successful empty reply to the FUSE client.
	pub fn ok_empty(self) -> Result<(), SendError<S::Error>> {
		self.inner.send_0(0)
	}

	/// Send an error reply to the FUSE client.
	pub fn err(
		self,
		error: impl Into<crate::Error>,
	) -> Result<(), SendError<S::Error>> {
		self.inner.send_0(error.into().0.get())
	}
}

#[allow(missing_docs)] // TODO
pub trait CuseHandlers {
	#[allow(missing_docs)] // TODO
	fn unimplemented(&self, request: CuseRequest<'_>);

	#[allow(missing_docs)] // TODO
	fn dispatch(&self, request: CuseRequest<'_>) {
		let opcode = request.header().opcode();
		if opcode == fuse_opcode::FUSE_READ {
			self.read(request);
			return;
		}
		if opcode == fuse_opcode::FUSE_WRITE {
			self.write(request);
			return;
		}
		match opcode {
			fuse_opcode::FUSE_FLUSH => self.flush(request),
			fuse_opcode::FUSE_FSYNC => self.fsync(request),
			fuse_opcode::FUSE_INTERRUPT => self.interrupt(request),
			fuse_opcode::FUSE_IOCTL => self.ioctl(request),
			fuse_opcode::FUSE_OPEN => self.open(request),
			fuse_opcode::FUSE_POLL => self.poll(request),
			fuse_opcode::FUSE_RELEASE => self.release(request),
			_ => self.unimplemented(request),
		}
	}

	/// Request handler for [`FUSE_FLUSH`](fuse_opcode::FUSE_FLUSH).
	fn flush(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_FSYNC`](fuse_opcode::FUSE_FSYNC).
	fn fsync(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_INTERRUPT`](fuse_opcode::FUSE_INTERRUPT).
	fn interrupt(&self, request: CuseRequest<'_>) {
		let _ = request;
	}

	/// Request handler for [`FUSE_IOCTL`](fuse_opcode::FUSE_IOCTL).
	fn ioctl(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_OPEN`](fuse_opcode::FUSE_OPEN).
	fn open(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_POLL`](fuse_opcode::FUSE_POLL).
	fn poll(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_READ`](fuse_opcode::FUSE_READ).
	fn read(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_RELEASE`](fuse_opcode::FUSE_RELEASE).
	fn release(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_WRITE`](fuse_opcode::FUSE_WRITE).
	fn write(&self, request: CuseRequest<'_>) {
		self.unimplemented(request)
	}
}

#[allow(missing_docs)] // TODO
pub trait FuseHandlers {
	#[allow(missing_docs)] // TODO
	fn unimplemented(&self, request: FuseRequest<'_>);

	#[allow(missing_docs)] // TODO
	fn dispatch(&self, request: FuseRequest<'_>) {
		let opcode = request.header().opcode();
		if opcode == fuse_opcode::FUSE_READ {
			self.read(request);
			return;
		}
		if opcode == fuse_opcode::FUSE_WRITE {
			self.write(request);
			return;
		}
		match opcode {
			fuse_opcode::FUSE_ACCESS => self.access(request),
			fuse_opcode::FUSE_BATCH_FORGET => self.batch_forget(request),
			fuse_opcode::FUSE_BMAP => self.bmap(request),
			fuse_opcode::FUSE_COPY_FILE_RANGE => self.copy_file_range(request),
			fuse_opcode::FUSE_CREATE => self.create(request),
			fuse_opcode::FUSE_DESTROY => self.destroy(request),
			fuse_opcode::FUSE_FALLOCATE => self.fallocate(request),
			fuse_opcode::FUSE_FLUSH => self.flush(request),
			fuse_opcode::FUSE_FORGET => self.forget(request),
			fuse_opcode::FUSE_FSYNC => self.fsync(request),
			fuse_opcode::FUSE_FSYNCDIR => self.fsyncdir(request),
			fuse_opcode::FUSE_GETATTR => self.getattr(request),
			fuse_opcode::FUSE_GETLK => self.getlk(request),
			fuse_opcode::FUSE_GETXATTR => self.getxattr(request),
			fuse_opcode::FUSE_INTERRUPT => self.interrupt(request),
			fuse_opcode::FUSE_IOCTL => self.ioctl(request),
			fuse_opcode::FUSE_LINK => self.link(request),
			fuse_opcode::FUSE_LISTXATTR => self.listxattr(request),
			fuse_opcode::FUSE_LOOKUP => self.lookup(request),
			fuse_opcode::FUSE_LSEEK => self.lseek(request),
			fuse_opcode::FUSE_MKDIR => self.mkdir(request),
			fuse_opcode::FUSE_MKNOD => self.mknod(request),
			fuse_opcode::FUSE_OPEN => self.open(request),
			fuse_opcode::FUSE_OPENDIR => self.opendir(request),
			fuse_opcode::FUSE_POLL => self.poll(request),
			fuse_opcode::FUSE_READDIR => self.readdir(request),
			fuse_opcode::FUSE_READDIRPLUS => self.readdirplus(request),
			fuse_opcode::FUSE_READLINK => self.readlink(request),
			fuse_opcode::FUSE_RELEASE => self.release(request),
			fuse_opcode::FUSE_RELEASEDIR => self.releasedir(request),
			fuse_opcode::FUSE_REMOVEXATTR => self.removexattr(request),
			fuse_opcode::FUSE_RENAME => self.rename(request),
			fuse_opcode::FUSE_RENAME2 => self.rename2(request),
			fuse_opcode::FUSE_RMDIR => self.rmdir(request),
			fuse_opcode::FUSE_SETATTR => self.setattr(request),
			fuse_opcode::FUSE_SETLK => self.setlk(request),
			fuse_opcode::FUSE_SETLKW => self.setlkw(request),
			fuse_opcode::FUSE_SETXATTR => self.setxattr(request),
			fuse_opcode::FUSE_STATFS => self.statfs(request),
			fuse_opcode::FUSE_SYMLINK => self.symlink(request),
			fuse_opcode::FUSE_SYNCFS => self.syncfs(request),
			fuse_opcode::FUSE_UNLINK => self.unlink(request),
			_ => self.unimplemented(request),
		}
	}

	/// Request handler for [`FUSE_ACCESS`](fuse_opcode::FUSE_ACCESS).
	fn access(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_BATCH_FORGET`].
	///
	/// [`FUSE_BATCH_FORGET`]: fuse_opcode::FUSE_BATCH_FORGET
	fn batch_forget(&self, request: FuseRequest<'_>) {
		let _ = request;
	}

	/// Request handler for [`FUSE_BMAP`](fuse_opcode::FUSE_BMAP).
	fn bmap(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_COPY_FILE_RANGE`].
	///
	/// [`FUSE_COPY_FILE_RANGE`]: fuse_opcode::FUSE_COPY_FILE_RANGE
	fn copy_file_range(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_CREATE`](fuse_opcode::FUSE_CREATE).
	fn create(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_DESTROY`](fuse_opcode::FUSE_DESTROY).
	fn destroy(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_FALLOCATE`](fuse_opcode::FUSE_FALLOCATE).
	fn fallocate(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_FLUSH`](fuse_opcode::FUSE_FLUSH).
	fn flush(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_FORGET`](fuse_opcode::FUSE_FORGET)
	fn forget(&self, request: FuseRequest<'_>) {
		let _ = request;
	}

	/// Request handler for [`FUSE_FSYNC`](fuse_opcode::FUSE_FSYNC).
	fn fsync(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_FSYNCDIR`](fuse_opcode::FUSE_FSYNCDIR).
	fn fsyncdir(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_GETATTR`](fuse_opcode::FUSE_GETATTR).
	fn getattr(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_GETLK`](fuse_opcode::FUSE_GETLK).
	fn getlk(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_GETXATTR`](fuse_opcode::FUSE_GETXATTR).
	fn getxattr(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_INTERRUPT`](fuse_opcode::FUSE_INTERRUPT).
	fn interrupt(&self, request: FuseRequest<'_>) {
		let _ = request;
	}

	/// Request handler for [`FUSE_IOCTL`](fuse_opcode::FUSE_IOCTL).
	fn ioctl(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_LINK`](fuse_opcode::FUSE_LINK).
	fn link(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_LISTXATTR`](fuse_opcode::FUSE_LISTXATTR).
	fn listxattr(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_LOOKUP`](fuse_opcode::FUSE_LOOKUP).
	fn lookup(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_LSEEK`](fuse_opcode::FUSE_LSEEK).
	fn lseek(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_MKDIR`](fuse_opcode::FUSE_MKDIR).
	fn mkdir(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_MKNOD`](fuse_opcode::FUSE_MKNOD).
	fn mknod(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_OPEN`](fuse_opcode::FUSE_OPEN).
	fn open(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_OPENDIR`](fuse_opcode::FUSE_OPENDIR).
	fn opendir(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_POLL`](fuse_opcode::FUSE_POLL).
	fn poll(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_READ`](fuse_opcode::FUSE_READ).
	fn read(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_READDIR`](fuse_opcode::FUSE_READDIR).
	fn readdir(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_READDIRPLUS`](fuse_opcode::FUSE_READDIRPLUS).
	fn readdirplus(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_READLINK`](fuse_opcode::FUSE_READLINK).
	fn readlink(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_RELEASE`](fuse_opcode::FUSE_RELEASE).
	fn release(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_RELEASEDIR`](fuse_opcode::FUSE_RELEASEDIR).
	fn releasedir(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_REMOVEXATTR`](fuse_opcode::FUSE_REMOVEXATTR).
	fn removexattr(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_RENAME`](fuse_opcode::FUSE_RENAME).
	fn rename(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_RENAME2`](fuse_opcode::FUSE_RENAME2).
	fn rename2(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_RMDIR`](fuse_opcode::FUSE_RMDIR).
	fn rmdir(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_SETATTR`](fuse_opcode::FUSE_SETATTR).
	fn setattr(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_SETLK`](fuse_opcode::FUSE_SETLK).
	fn setlk(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_SETLKW`](fuse_opcode::FUSE_SETLKW).
	fn setlkw(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_SETXATTR`](fuse_opcode::FUSE_SETXATTR).
	fn setxattr(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_STATFS`](fuse_opcode::FUSE_STATFS).
	fn statfs(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_SYMLINK`](fuse_opcode::FUSE_SYMLINK).
	fn symlink(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_SYNCFS`](fuse_opcode::FUSE_SYNCFS).
	fn syncfs(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_UNLINK`](fuse_opcode::FUSE_UNLINK).
	fn unlink(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}

	/// Request handler for [`FUSE_WRITE`](fuse_opcode::FUSE_WRITE).
	fn write(&self, request: FuseRequest<'_>) {
		self.unimplemented(request)
	}
}

/// Represents an active connection to a CUSE client.
pub struct CuseConnection<S> {
	socket: S,
	layout: CuseLayout,
	recv_buf_len: usize,
}

impl<S: CuseSocket> CuseConnection<S> {
	/// Perform a CUSE connection handshake.
	///
	/// When a CUSE connection is being established the client will send a
	/// [`CuseInitRequest`] and the server responds with a [`CuseInitResponse`].
	///
	/// The reply specifies the name and device number of the CUSE device
	/// that will be created for this server.
	pub fn connect<F>(
		socket: S,
		device_name: &CuseDeviceName,
		device_number: CuseDeviceNumber,
		mut init_fn: F,
	) -> Result<CuseConnection<S>, ServerError<S::Error>>
	where
		F: FnMut(&CuseInitRequest, &mut CuseInitResponse),
	{
		let mut buf = crate::io::MinReadBuffer::new();
		let layout = CuseLayout {
			version_minor: kernel::FUSE_KERNEL_MINOR_VERSION as u16,
		};

		loop {
			let recv_len = socket.recv(buf.as_slice_mut())?;
			let recv_buf = buf.as_aligned_slice().truncate(recv_len);
			let request = CuseRequest::new(recv_buf, layout)?;
			let init_req = CuseInitRequest::try_from(request)?;

			let (reply, ok) = cuse_handshake(&init_req, || {
				let mut reply = CuseInitResponse::new(device_name);
				reply.set_device_number(device_number);
				init_fn(&init_req, &mut reply);
				reply
			})?;

			let request_id = request.header().request_id();
			reply.send_to(CuseReplySender {
				inner: ReplySender {
					socket: &socket,
					request_id: request_id.get(),
				},
			})?;

			if !ok {
				continue;
			}

			return Ok(Self {
				socket,
				layout: CuseLayout {
					version_minor: reply.raw.minor as u16,
				},
				recv_buf_len: recv_buf_len(reply.max_write()),
			});
		}
	}

	/// Receive a CUSE request from the client.
	pub fn recv<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<CuseRequest<'a>, ServerError<S::Error>> {
		use crate::io::AlignedSlice;
		let recv_len = self.socket.recv(buf.get_mut())?;
		let recv_buf = AlignedSlice::from(buf).truncate(recv_len);
		Ok(CuseRequest::new(recv_buf, self.layout)?)
	}

	#[allow(missing_docs)] // TODO
	pub fn reply(&self, request_id: NonZeroU64) -> CuseReplySender<'_, S> {
		CuseReplySender {
			inner: ReplySender {
				socket: &self.socket,
				request_id: request_id.get(),
			},
		}
	}

	/*
	TODO

	pub fn notify(
		&self,
		notification: &crate::FuseNotification<'_>,
	) -> Result<(), SendError<S::Error>> {
		let mut header = crate::ResponseHeader::new_notification();
		self.socket.send(notification.encode(&mut header))
	}
	*/
}

impl<S> CuseConnection<S> {
	/// Returns a reference to the underlying [`Socket`] for this connection.
	#[inline]
	#[must_use]
	pub fn socket(&self) -> &S {
		&self.socket
	}

	#[allow(missing_docs)] // TODO
	#[inline]
	#[must_use]
	pub fn layout(&self) -> CuseLayout {
		self.layout
	}

	/// Returns the minimum size of the receive buffer for this connection.
	#[inline]
	#[must_use]
	pub fn recv_buf_len(&self) -> usize {
		self.recv_buf_len
	}
}

pub(crate) fn cuse_handshake<'a, E, F>(
	request: &CuseInitRequest,
	mut new_reply: F,
) -> Result<(CuseInitResponse<'a>, bool), ServerError<E>>
where
	F: FnMut() -> CuseInitResponse<'a>,
{
	match negotiate_version(request.version()) {
		Some(version) => {
			let mut reply = new_reply();
			reply.set_version(version);
			Ok((reply, true))
		},
		None => {
			let mut reply = CuseInitResponse::new_nameless();
			reply.set_version(crate::Version::LATEST);
			Ok((reply, false))
		},
	}
}

/// Represents an active connection to a FUSE client.
pub struct FuseConnection<S> {
	socket: S,
	layout: FuseLayout,
	recv_buf_len: usize,
}

impl<S: FuseSocket> FuseConnection<S> {
	/// Perform a FUSE connection handshake.
	///
	/// When a FUSE session is being established the client will send a
	/// [`FuseInitRequest`] and the server responds with a [`FuseInitResponse`].
	///
	/// The reply specifies tunable parameters and optional features of the
	/// filesystem server.
	pub fn connect<F>(
		socket: S,
		mut init_fn: F,
	) -> Result<FuseConnection<S>, ServerError<S::Error>>
	where
		F: FnMut(&FuseInitRequest, &mut FuseInitResponse),
	{
		let mut buf = crate::io::MinReadBuffer::new();
		let layout = FuseLayout {
			version_minor: kernel::FUSE_KERNEL_MINOR_VERSION as u16,
			features: 0,
		};

		loop {
			let recv_len = socket.recv(buf.as_slice_mut())?;
			let recv_buf = buf.as_aligned_slice().truncate(recv_len);
			let request = FuseRequest::new(recv_buf, layout)?;
			let init_req = FuseInitRequest::try_from(request)?;

			let (reply, ok) = fuse_handshake(&init_req, || {
				let mut reply = FuseInitResponse::new();
				init_fn(&init_req, &mut reply);
				reply
			})?;

			let request_id = request.header().request_id();
			reply.raw.send_to(FuseReplySender {
				inner: ReplySender {
					socket: &socket,
					request_id: request_id.get(),
				},
				layout,
			})?;

			if !ok {
				continue;
			}

			return Ok(Self {
				socket,
				layout: FuseLayout::new2(&reply.raw),
				recv_buf_len: recv_buf_len(reply.max_write()),
			});
		}
	}

	/// Receive a FUSE request from the client.
	pub fn recv<'a>(
		&self,
		mut buf: crate::io::AlignedSliceMut<'a>,
	) -> Result<Option<FuseRequest<'a>>, ServerError<S::Error>> {
		use crate::io::AlignedSlice;
		let recv_len = match self.socket.recv(buf.get_mut()) {
			Ok(len) => len,
			Err(RecvError::ConnectionClosed(_)) => return Ok(None),
			Err(err) => return Err(err.into()),
		};
		let recv_buf = AlignedSlice::from(buf).truncate(recv_len);
		Ok(Some(FuseRequest::new(recv_buf, self.layout)?))
	}

	#[allow(missing_docs)] // TODO
	pub fn reply(&self, request_id: NonZeroU64) -> FuseReplySender<'_, S> {
		FuseReplySender {
			inner: ReplySender {
				socket: &self.socket,
				request_id: request_id.get(),
			},
			layout: self.layout,
		}
	}

	#[allow(missing_docs)] // TODO
	pub fn notify(
		&self,
		notification: &crate::FuseNotification<'_>,
	) -> Result<(), SendError<S::Error>> {
		let mut header = crate::ResponseHeader::new_notification();
		self.socket.send(notification.encode(&mut header))
	}
}

impl<S> FuseConnection<S> {
	/// Returns a reference to the underlying [`Socket`] for this connection.
	#[inline]
	#[must_use]
	pub fn socket(&self) -> &S {
		&self.socket
	}

	#[allow(missing_docs)] // TODO
	#[inline]
	#[must_use]
	pub fn layout(&self) -> FuseLayout {
		self.layout
	}

	/// Returns the minimum size of the receive buffer for this connection.
	///
	/// This value is computed from `max_write`. Operations with their own
	/// notion of maximum size, such as `FUSE_SETXATTR`, may require a receive
	/// buffer length greater than this value.
	#[inline]
	#[must_use]
	pub fn recv_buf_len(&self) -> usize {
		self.recv_buf_len
	}
}

pub(crate) fn fuse_handshake<E, F>(
	request: &FuseInitRequest,
	mut new_reply: F,
) -> Result<(FuseInitResponse, bool), ServerError<E>>
where
	F: FnMut() -> FuseInitResponse,
{
	match negotiate_version(request.version()) {
		Some(version) => {
			let mut reply = new_reply();
			reply.set_version(version);
			Ok((reply, true))
		},
		None => {
			let mut reply = FuseInitResponse::new();
			reply.set_version(crate::Version::LATEST);
			Ok((reply, false))
		},
	}
}

/// Builder for CUSE connections.
pub struct CuseServer<'a> {
	device_name: &'a CuseDeviceName,
	device_number: CuseDeviceNumber,
	flags: CuseInitFlags,
	max_read: u32,
	max_write: u32,
}

impl<'a> CuseServer<'a> {
	/// Create a new `CuseServer` with the given device name and device number.
	#[must_use]
	pub fn new(
		device_name: &'a CuseDeviceName,
		device_number: CuseDeviceNumber,
	) -> CuseServer<'a> {
		Self {
			device_name,
			device_number,
			flags: CuseInitFlags::new(),
			max_read: 0,
			max_write: 0,
		}
	}

	/// Establish a new CUSE connection by on the given socket.
	pub fn connect<S: CuseSocket>(
		&self,
		socket: S,
	) -> Result<CuseConnection<S>, ServerError<S::Error>> {
		CuseConnection::connect(
			socket,
			self.device_name,
			self.device_number,
			|request, reply| {
				reply.set_max_read(self.max_read);
				reply.set_max_write(self.max_write);
				reply.set_flags(request.flags() & self.flags);
			},
		)
	}

	/// Set the connection's [`max_read`].
	///
	/// [`max_read`]: CuseInitResponse::max_read
	pub fn max_read(&mut self, max_read: u32) -> &mut Self {
		self.max_read = max_read;
		self
	}

	/// Set the connection's [`max_write`].
	///
	/// [`max_write`]: CuseInitResponse::max_write
	pub fn max_write(&mut self, max_write: u32) -> &mut Self {
		self.max_write = max_write;
		self
	}

	/// Adjust which [`CuseInitFlags`] the server will offer.
	///
	/// Init flags will be enabled if they are offered by the server and
	/// supported by the client.
	pub fn update_flags(
		&mut self,
		f: impl FnOnce(&mut CuseInitFlags),
	) -> &mut Self {
		f(&mut self.flags);
		self
	}

	/// Offer the [`UNRESTRICTED_IOCTL`] init flag.
	///
	/// [`UNRESTRICTED_IOCTL`]: CuseInitFlag::UNRESTRICTED_IOCTL
	pub fn enable_unrestricted_ioctl(&mut self) -> &mut Self {
		self.update_flags(|flags| {
			flags.set(CuseInitFlag::UNRESTRICTED_IOCTL);
		});
		self
	}
}

/// Builder for FUSE connections.
pub struct FuseServer {
	init_reply: FuseInitResponse,
}

impl FuseServer {
	/// Create a new `FuseServer`.
	#[must_use]
	pub fn new() -> FuseServer {
		Self {
			init_reply: FuseInitResponse::new(),
		}
	}

	/// Establish a new FUSE connection by on the given socket.
	pub fn connect<S: FuseSocket>(
		&self,
		socket: S,
	) -> Result<FuseConnection<S>, ServerError<S::Error>> {
		let opts = &self.init_reply;
		FuseConnection::connect(socket, |request, reply| {
			reply.set_congestion_threshold(opts.congestion_threshold());
			reply.set_max_background(opts.max_background());
			reply.set_max_readahead(opts.max_readahead());
			reply.set_max_write(opts.max_write());
			reply.set_time_granularity(opts.time_granularity());
			reply.set_flags(request.flags() & opts.flags());
		})
	}

	/// Set the connection's [`congestion_threshold`].
	///
	/// [`congestion_threshold`]: FuseInitResponse::congestion_threshold
	pub fn congestion_threshold(
		&mut self,
		congestion_threshold: u16,
	) -> &mut Self {
		self.init_reply.set_congestion_threshold(congestion_threshold);
		self
	}

	/// Set the connection's [`max_background`].
	///
	/// [`max_background`]: FuseInitResponse::max_background
	pub fn max_background(&mut self, max_background: u16) -> &mut Self {
		self.init_reply.set_max_background(max_background);
		self
	}

	/// Set the connection's [`max_readahead`].
	///
	/// [`max_readahead`]: FuseInitResponse::max_readahead
	pub fn max_readahead(&mut self, max_readahead: u32) -> &mut Self {
		self.init_reply.set_max_readahead(max_readahead);
		self
	}

	/// Set the connection's [`max_write`].
	///
	/// If `max_write` is greater than 4096 this method also offers the
	/// [`BIG_WRITES`] init flag.
	///
	/// [`max_write`]: FuseInitResponse::max_write
	/// [`BIG_WRITES`]: FuseInitFlag::BIG_WRITES
	pub fn max_write(&mut self, max_write: u32) -> &mut Self {
		self.init_reply.set_max_write(max_write);
		if max_write > 4096 {
			self.init_reply.update_flags(|flags| {
				flags.set(FuseInitFlag::BIG_WRITES);
			});
		}
		self
	}

	/// Set the connection's [`time_granularity`].
	///
	/// [`time_granularity`]: FuseInitResponse::time_granularity
	pub fn time_granularity(&mut self, time_granularity: u32) -> &mut Self {
		self.init_reply.set_time_granularity(time_granularity);
		self
	}

	/// Adjust which [`FuseInitFlags`] the server will offer.
	///
	/// Init flags will be enabled if they are offered by the server and
	/// supported by the client.
	pub fn update_flags(
		&mut self,
		f: impl FnOnce(&mut FuseInitFlags),
	) -> &mut Self {
		self.init_reply.update_flags(f);
		self
	}
}

// }}}

fn negotiate_version(kernel: crate::Version) -> Option<crate::Version> {
	if kernel.major() != crate::Version::LATEST.major() {
		// TODO: hard error on kernel major version < FUSE_KERNEL_VERSION
		return None;
	}
	Some(crate::Version::new(
		crate::Version::LATEST.major(),
		cmp::min(kernel.minor(), crate::Version::LATEST.minor()),
	))
}

fn recv_buf_len(max_write: u32) -> usize {
	const FUSE_BUFFER_HEADER_SIZE: usize = 4096;
	cmp::max(
		(max_write as usize).saturating_add(FUSE_BUFFER_HEADER_SIZE),
		crate::io::MinReadBuffer::LEN,
	)
}

/// Serve CUSE requests in a loop, in a single thread without allocating.
pub fn cuse_serve_local<S: CuseSocket>(
	conn: &CuseConnection<S>,
	handlers: &impl CuseHandlers,
	buf: &mut impl crate::io::AsAlignedSliceMut,
) -> Result<(), ServerError<S::Error>> {
	loop {
		let request = conn.recv(buf.as_aligned_slice_mut())?;
		handlers.dispatch(request);
	}
}

/// Serve FUSE requests in a loop, in a single thread without allocating.
///
/// Returns `Ok(())` when the connection is closed, such as by the user
/// unmounting the filesystem with `fusermount -u`.
pub fn fuse_serve_local<S: FuseSocket>(
	conn: &FuseConnection<S>,
	handlers: &impl FuseHandlers,
	buf: &mut impl crate::io::AsAlignedSliceMut,
) -> Result<(), ServerError<S::Error>> {
	while let Some(request) = conn.recv(buf.as_aligned_slice_mut())? {
		handlers.dispatch(request);
	}
	Ok(())
}
