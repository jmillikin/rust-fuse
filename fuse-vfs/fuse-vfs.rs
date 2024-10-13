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

#![allow(
	clippy::new_without_default,
)]

#![warn(
	// API hygiene
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::must_use_candidate,

	// Panic hygiene
	clippy::expect_used,
	clippy::todo,
	clippy::unimplemented,
	clippy::unwrap_used,

	// Documentation coverage
	missing_docs,
	clippy::missing_panics_doc,

	// Explicit casts
	clippy::fn_to_numeric_cast_any,
	clippy::ptr_as_ptr,

	// Optimization
	clippy::trivially_copy_pass_by_ref,

	// Unused symbols
	clippy::let_underscore_must_use,
	clippy::no_effect_underscore_binding,
	clippy::used_underscore_binding,

	// Leftover debugging
	clippy::print_stderr,
	clippy::print_stdout,
)]

// FIXME
#![allow(
	clippy::missing_panics_doc,
	missing_docs,
)]

use std::borrow::Cow;
use std::cmp;
use std::collections::HashMap;
use std::ffi::CStr;
use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use fuse::{
	Error,
	NodeId,
	NodeName,
	RequestHeader,
};
use fuse::kernel;
use fuse::os::OsError;
use fuse::server;
use fuse::server::{
	FuseRequest,
	ServerError,
};

// Node {{{

#[allow(unused_variables)]
pub trait Node: Send + Sync {
	fn as_directory(&self) -> Option<&dyn Directory> {
		None
	}

	fn as_file(&self) -> Option<&dyn File> {
		None
	}

	fn as_symlink(&self) -> Option<&dyn Symlink> {
		None
	}

	fn getattr(
		&self,
		header: &RequestHeader,
		request: server::GetattrRequest<'_>,
	) -> Result<GetattrResult, Error>;

	fn getxattr(
		&self,
		header: &RequestHeader,
		request: server::GetxattrRequest<'_>,
	) -> Result<GetxattrResult, Error> {
		Err(OsError::XATTR_NOT_FOUND)
	}

	fn listxattr(
		&self,
		header: &RequestHeader,
		request: server::ListxattrRequest<'_>,
	) -> Result<ListxattrResult, Error> {
		Ok(ListxattrResult::names(b""))
	}

	fn statfs(
		&self,
		header: &RequestHeader,
		request: server::StatfsRequest<'_>,
	) -> Result<StatfsResult, Error> {
		Ok(StatfsResult::new(fuse::StatfsAttributes::new()))
	}
}

pub struct GetattrResult {
	node_attr: fuse::NodeAttr,
	pub cache_timeout: Duration,
}

impl GetattrResult {
	#[must_use]
	pub fn new(node_attr: fuse::NodeAttr) -> GetattrResult {
		GetattrResult {
			node_attr,
			cache_timeout: Duration::ZERO,
		}
	}
}

pub struct GetxattrResult {
	buf: Cow<'static, [u8]>,
}

impl GetxattrResult {
	#[must_use]
	pub fn size(size: u32) -> GetxattrResult {
		let mut reply = fuse::kernel::fuse_getxattr_out::new();
		reply.size = size;
		Self {
			buf: Cow::Owned(reply.as_bytes().to_owned()),
		}
	}

	#[must_use]
	pub fn value(value: impl Into<Cow<'static, [u8]>>) -> GetxattrResult {
		GetxattrResult { buf: value.into() }
	}
}

pub struct ListxattrResult {
	buf: Cow<'static, [u8]>,
}

impl ListxattrResult {
	#[must_use]
	pub fn size(size: u32) -> ListxattrResult {
		let mut reply = fuse::kernel::fuse_getxattr_out::new();
		reply.size = size;
		Self {
			buf: Cow::Owned(reply.as_bytes().to_owned()),
		}
	}

	#[must_use]
	pub fn names(names: impl Into<Cow<'static, [u8]>>) -> ListxattrResult {
		ListxattrResult { buf: names.into() }
	}
}

pub struct StatfsResult {
	statfs_attr: fuse::StatfsAttributes,
}

impl StatfsResult {
	#[must_use]
	pub fn new(statfs_attr: fuse::StatfsAttributes) -> StatfsResult {
		StatfsResult { statfs_attr }
	}
}

// Node }}}

// Directory {{{

pub trait Directory: Node {
	fn lookup(
		&self,
		header: &RequestHeader,
		request: server::LookupRequest<'_>,
	) -> Result<LookupResult, Error>;

	fn opendir(
		&self,
		header: &RequestHeader,
		request: server::OpendirRequest<'_>,
	) -> Result<OpendirResult, Error>;
}

pub struct LookupResult {
	node: Option<(Arc<dyn Node>, fuse::NodeAttr)>,
	pub generation: u64,
	pub entry_cache_timeout: Duration,
	pub attr_cache_timeout: Duration,
}

impl LookupResult {
	#[must_use]
	pub fn found(
		node: Arc<dyn Node>,
		node_attr: fuse::NodeAttr,
	) -> LookupResult {
		Self {
			node: Some((node, node_attr)),
			generation: 0,
			entry_cache_timeout: Duration::ZERO,
			attr_cache_timeout: Duration::ZERO,
		}
	}

	#[must_use]
	pub fn not_found() -> LookupResult {
		Self {
			node: None,
			generation: 0,
			entry_cache_timeout: Duration::ZERO,
			attr_cache_timeout: Duration::ZERO,
		}
	}
}

pub struct OpendirResult {
	handle: Arc<dyn DirectoryHandle>,
	pub open_flags: fuse::OpenFlags,
}

impl OpendirResult {
	#[must_use]
	pub fn new(handle: Arc<dyn DirectoryHandle>) -> OpendirResult {
		Self {
			handle,
			open_flags: 0,
		}
	}
}

// Directory }}}

// DirectoryHandle {{{

pub trait DirectoryHandle: Send + Sync {
	fn as_readdir_handle(&self) -> Option<&dyn ReaddirHandle> {
		None
	}

	fn as_readdirplus_handle(&self) -> Option<&dyn ReaddirplusHandle> {
		None
	}

	#[allow(unused_variables)]
	fn releasedir(
		&self,
		header: &RequestHeader,
		request: server::ReleasedirRequest<'_>,
	) -> Result<ReleasedirResult, Error> {
		Ok(ReleasedirResult::new())
	}
}

#[non_exhaustive]
pub struct ReleasedirResult {}

impl ReleasedirResult {
	#[must_use]
	pub fn new() -> ReleasedirResult {
		Self {}
	}
}

pub trait ReaddirHandle: DirectoryHandle {
	fn readdir(
		&self,
		header: &RequestHeader,
		request: server::ReaddirRequest<'_>,
	) -> Result<ReaddirResult, Error>;
}

pub struct ReaddirResult {
	entries: Cow<'static, [u8]>,
}

impl ReaddirResult {
	#[must_use]
	pub fn new(entries: impl Into<Cow<'static, [u8]>>) -> ReaddirResult {
		Self {
			entries: entries.into(),
		}
	}
}

pub trait ReaddirplusHandle: ReaddirHandle {
	fn readdirplus(
		&self,
		header: &RequestHeader,
		request: server::ReaddirplusRequest<'_>,
	) -> Result<ReaddirplusResult, Error>;
}

pub struct ReaddirplusResult {
	entries: Cow<'static, [u8]>,
}

impl ReaddirplusResult {
	#[must_use]
	pub fn new(entries: impl Into<Cow<'static, [u8]>>) -> ReaddirplusResult {
		Self {
			entries: entries.into(),
		}
	}
}

// DirectoryHandle }}}

// File {{{

pub trait File: Node {
	fn open(
		&self,
		header: &RequestHeader,
		request: server::OpenRequest<'_>,
	) -> Result<OpenResult, Error>;
}

pub struct OpenResult {
	handle: Arc<dyn FileHandle>,
	pub open_flags: fuse::OpenFlags,
}

impl OpenResult {
	#[must_use]
	pub fn new(handle: Arc<dyn FileHandle>) -> OpenResult {
		Self {
			handle,
			open_flags: 0,
		}
	}
}

// File }}}

// FileHandle {{{

pub trait FileHandle: Send + Sync {
	fn as_read_handle(&self) -> Option<&dyn ReadHandle> {
		None
	}

	fn as_write_handle(&self) -> Option<&dyn WriteHandle> {
		None
	}

	#[allow(unused_variables)]
	fn release(
		&self,
		header: &RequestHeader,
		request: server::ReleaseRequest<'_>,
	) -> Result<ReleaseResult, Error> {
		Ok(ReleaseResult::new())
	}
}

#[non_exhaustive]
pub struct ReleaseResult {}

impl ReleaseResult {
	#[must_use]
	pub fn new() -> ReleaseResult {
		Self {}
	}
}

pub trait ReadHandle: FileHandle {
	fn read(
		&self,
		header: &RequestHeader,
		request: server::ReadRequest<'_>,
	) -> Result<ReadResult, Error>;
}

pub struct ReadResult {
	data: Cow<'static, [u8]>,
}

impl ReadResult {
	#[must_use]
	pub fn new(data: impl Into<Cow<'static, [u8]>>) -> ReadResult {
		Self { data: data.into() }
	}
}

pub trait WriteHandle: FileHandle {
	fn write(
		&self,
		header: &RequestHeader,
		request: server::WriteRequest<'_>,
	) -> Result<WriteResult, Error>;
}

pub struct WriteResult {
	size: u32,
}

impl WriteResult {
	#[must_use]
	pub fn new(size: u32) -> WriteResult {
		Self { size }
	}
}

// FileHandle }}}

// Symlink {{{

pub trait Symlink: Node {
	fn readlink(
		&self,
		header: &RequestHeader,
		request: server::ReadlinkRequest<'_>,
	) -> Result<ReadlinkResult, Error>;
}

pub struct ReadlinkResult {
	target: Cow<'static, CStr>,
}

impl ReadlinkResult {
	#[must_use]
	pub fn new(target: impl Into<Cow<'static, CStr>>) -> ReadlinkResult {
		Self {
			target: target.into(),
		}
	}
}

// Symlink }}}

// Filesystem {{{

pub struct Filesystem<'a, S> {
	conn: &'a server::FuseConnection<S>,
	nodes: Arc<NodeMap>,
	handles: Arc<RwLock<HandlesMap>>,
}

impl<'a, S> Filesystem<'a, S> {
	#[must_use]
	pub fn new(
		conn: &'a server::FuseConnection<S>,
		root: Arc<dyn Node>,
	) -> Filesystem<'a, S> {
		Self {
			conn,
			nodes: Arc::new(NodeMap::new(root)),
			handles: Arc::new(RwLock::new(HandlesMap::new())),
		}
	}
}

impl<S: fuse::server::FuseSocket> Filesystem<'_, S> {
	#[must_use]
	pub fn dispatch(
		&self,
		request: FuseRequest<'_>,
	) -> Option<Result<(), ServerError<S::Error>>> {
		use kernel::fuse_opcode as op;
		let opcode = request.header().opcode();
		if opcode == op::FUSE_READ {
			return Some(self.read(request));
		}
		if opcode == op::FUSE_WRITE {
			return Some(self.write(request));
		}
		Some(match request.header().opcode() {
			op::FUSE_BATCH_FORGET => self.forget(request),
			op::FUSE_FORGET => self.forget(request),
			op::FUSE_GETATTR => self.getattr(request),
			op::FUSE_GETXATTR => self.getxattr(request),
			op::FUSE_LISTXATTR => self.listxattr(request),
			op::FUSE_LOOKUP => self.lookup(request),
			op::FUSE_OPEN => self.open(request),
			op::FUSE_OPENDIR => self.opendir(request),
			op::FUSE_READDIR => self.readdir(request),
			op::FUSE_READDIRPLUS => self.readdirplus(request),
			op::FUSE_READLINK => self.readlink(request),
			op::FUSE_RELEASE => self.release(request),
			op::FUSE_RELEASEDIR => self.releasedir(request),
			op::FUSE_STATFS => self.statfs(request),
			_ => return None,
		})
	}

	fn forget(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let request = server::ForgetRequest::try_from(request)?;
		self.nodes.forget(request.items());
		Ok(())
	}

	fn getattr(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::GetattrRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let result = match node.getattr(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};

		let mut reply = kernel::fuse_attr_out::new();
		reply.attr = *result.node_attr.raw();
		split_duration(
			result.cache_timeout,
			&mut reply.attr_valid,
			&mut reply.attr_valid_nsec,
		);
		Ok(send_reply.ok(&reply)?)
	}

	fn getxattr(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::GetxattrRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let result = match node.getxattr(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		Ok(send_reply.ok_buf(&result.buf)?)
	}

	fn listxattr(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ListxattrRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let result = match node.listxattr(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		Ok(send_reply.ok_buf(&result.buf)?)
	}

	fn lookup(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::LookupRequest::try_from(request)?;

		let parent = match self.nodes.get(request.parent_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let parent_dir = match parent.as_directory() {
			Some(dir) => dir,
			None => return Ok(send_reply.err(OsError::NOT_DIRECTORY)?),
		};

		let mut reply = kernel::fuse_entry_out::new();
		let result = match parent_dir.lookup(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};

		split_duration(
			result.entry_cache_timeout,
			&mut reply.entry_valid,
			&mut reply.entry_valid_nsec,
		);

		if let Some((node, node_attr)) = result.node {
			let node_id = node_attr.node_id();
			reply.nodeid = node_id.get();
			reply.attr = *node_attr.raw();
			reply.generation = result.generation;
			split_duration(
				result.attr_cache_timeout,
				&mut reply.attr_valid,
				&mut reply.attr_valid_nsec,
			);
			send_reply.ok(&reply)?;
			self.nodes.add(node_id, node);
			return Ok(());
		}

		Ok(send_reply.ok(&reply)?)
	}

	fn open(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::OpenRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let Some(file) = node.as_file() else {
			return Ok(send_reply.err(OsError::NOT_SUPPORTED)?);
		};

		let result = match file.open(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};

		let handle_id = {
			#[allow(clippy::unwrap_used)]
			self.handles.write().unwrap().open_file(result.handle)
		};

		let mut reply = kernel::fuse_open_out::new();
		reply.fh = handle_id;
		reply.open_flags = result.open_flags;
		Ok(send_reply.ok(&reply)?)
	}

	fn opendir(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::OpendirRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};

		let dir = match node.as_directory() {
			Some(dir) => dir,
			None => return Ok(send_reply.err(OsError::NOT_DIRECTORY)?),
		};

		let result = match dir.opendir(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};

		let handle_id = {
			#[allow(clippy::unwrap_used)]
			self.handles.write().unwrap().open_dir(result.handle)
		};

		let mut reply = kernel::fuse_open_out::new();
		reply.fh = handle_id;
		reply.open_flags = result.open_flags;
		Ok(send_reply.ok(&reply)?)
	}

	fn read(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ReadRequest::try_from(request)?;

		let got_file_handle = {
			#[allow(clippy::unwrap_used)]
			let handles = self.handles.read().unwrap();
			handles.get_file(request.node_id(), request.handle()).cloned()
		};
		let file_handle = match got_file_handle {
			Ok(handle) => handle,
			Err(err) => return Ok(send_reply.err(err)?),
		};

		let handle = match file_handle.as_read_handle() {
			Some(handle) => handle,
			None => return Ok(send_reply.err(OsError::NOT_SUPPORTED)?),
		};
		let result = match handle.read(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		Ok(send_reply.ok_buf(&result.data)?)
	}

	fn readdir(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ReaddirRequest::try_from(request)?;

		let got_dir_handle = {
			#[allow(clippy::unwrap_used)]
			let handles = self.handles.read().unwrap();
			handles.get_dir(request.node_id(), request.handle()).cloned()
		};
		let dir_handle = match got_dir_handle {
			Ok(handle) => handle,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let handle = match dir_handle.as_readdir_handle() {
			Some(handle) => handle,
			None => return Ok(send_reply.err(OsError::NOT_SUPPORTED)?),
		};
		let result = match handle.readdir(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		Ok(send_reply.ok_buf(&result.entries)?)
	}

	fn readdirplus(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ReaddirplusRequest::try_from(request)?;

		let got_dir_handle = {
			#[allow(clippy::unwrap_used)]
			let handles = self.handles.read().unwrap();
			handles.get_dir(request.node_id(), request.handle()).cloned()
		};
		let dir_handle = match got_dir_handle {
			Ok(handle) => handle,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let handle = match dir_handle.as_readdirplus_handle() {
			Some(handle) => handle,
			None => return Ok(send_reply.err(OsError::NOT_SUPPORTED)?),
		};
		let result = match handle.readdirplus(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		Ok(send_reply.ok_buf(&result.entries)?)
	}

	fn readlink(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ReadlinkRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let symlink = match node.as_symlink() {
			Some(symlink) => symlink,
			None => return Ok(send_reply.err(OsError::NOT_SUPPORTED)?),
		};
		let result = match symlink.readlink(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		Ok(send_reply.ok_buf(result.target.to_bytes())?)
	}

	fn release(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ReleaseRequest::try_from(request)?;
		let node_id = request.node_id();
		let handle_id = request.handle();

		#[allow(clippy::unwrap_used)]
		let mut handles = self.handles.write().unwrap();
		let file_handle = match handles.get_file(node_id, handle_id) {
			Ok(handle) => handle,
			Err(err) => {
				core::mem::drop(handles);
				return Ok(send_reply.err(err)?);
			},
		};
		if let Err(err) = file_handle.release(header, request) {
			core::mem::drop(handles);
			return Ok(send_reply.err(err)?);
		};
		handles.close_file(node_id, handle_id);
		core::mem::drop(handles);
		Ok(send_reply.ok_empty()?)
	}

	fn releasedir(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::ReleasedirRequest::try_from(request)?;
		let node_id = request.node_id();
		let handle_id = request.handle();

		#[allow(clippy::unwrap_used)]
		let mut handles = self.handles.write().unwrap();
		let dir_handle = match handles.get_dir(node_id, handle_id) {
			Ok(handle) => handle,
			Err(err) => {
				core::mem::drop(handles);
				return Ok(send_reply.err(err)?);
			},
		};
		if let Err(err) = dir_handle.releasedir(header, request) {
			core::mem::drop(handles);
			return Ok(send_reply.err(err)?);
		};
		handles.close_dir(node_id, handle_id);
		core::mem::drop(handles);
		Ok(send_reply.ok_empty()?)
	}

	fn statfs(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::StatfsRequest::try_from(request)?;

		let node = match self.nodes.get(request.node_id()) {
			Ok(node) => node,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let result = match node.statfs(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let mut reply = fuse::kernel::fuse_statfs_out::new();
		reply.st = *result.statfs_attr.raw();
		Ok(send_reply.ok(&reply)?)
	}

	fn write(
		&self,
		request: FuseRequest<'_>,
	) -> Result<(), ServerError<S::Error>> {
		let send_reply = self.conn.reply(request.id());
		let header = request.header();
		let request = server::WriteRequest::try_from(request)?;

		let got_file_handle = {
			#[allow(clippy::unwrap_used)]
			let handles = self.handles.read().unwrap();
			handles.get_file(request.node_id(), request.handle()).cloned()
		};
		let file_handle = match got_file_handle {
			Ok(handle) => handle,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let handle = match file_handle.as_write_handle() {
			Some(handle) => handle,
			None => return Ok(send_reply.err(OsError::NOT_SUPPORTED)?),
		};
		let result = match handle.write(header, request) {
			Ok(result) => result,
			Err(err) => return Ok(send_reply.err(err)?),
		};
		let mut reply = fuse::kernel::fuse_write_out::new();
		reply.size = result.size;
		Ok(send_reply.ok(&reply)?)
	}
}

// Filesystem }}}

// NodeMap {{{

struct NodeMap {
	nodes: RwLock<HashMap<NodeId, NodeLookup>>,
}

struct NodeLookup {
	node: Arc<dyn Node>,
	lookup_count: u64,
}

impl NodeMap {
	fn new(root: Arc<dyn Node>) -> NodeMap {
		let mut nodes = HashMap::new();
		nodes.insert(NodeId::ROOT, NodeLookup {
			node: root,
			lookup_count: 0,
		});
		Self {
			nodes: RwLock::new(nodes),
		}
	}

	fn add(&self, node_id: NodeId, node: Arc<dyn Node>) {
		use std::collections::hash_map::Entry;

		#[allow(clippy::unwrap_used)]
		let mut nodes = self.nodes.write().unwrap();
		match nodes.entry(node_id) {
			Entry::Occupied(mut entry) => {
				entry.get_mut().lookup_count += 1;
			},
			Entry::Vacant(entry) => {
				entry.insert(NodeLookup {
					node,
					lookup_count: 1,
				});
			},
		}
	}

	fn forget(
		&self,
		forgets: impl Iterator<Item = fuse::server::ForgetRequestItem>,
	) {
		#[allow(clippy::unwrap_used)]
		let mut nodes = self.nodes.write().unwrap();
		for forget in forgets {
			let node_id = forget.node_id();
			if let Some(entry) = nodes.get_mut(&node_id) {
				let new_count = entry.lookup_count.saturating_sub(1);
				if new_count == 0 {
					nodes.remove(&node_id);
				} else {
					entry.lookup_count = new_count;
				}
			}
		}
	}

	fn get(&self, node_id: NodeId) -> Result<Arc<dyn Node>, Error> {
		#[allow(clippy::unwrap_used)]
		let nodes = self.nodes.read().unwrap();
		match nodes.get(&node_id) {
			Some(entry) => Ok(entry.node.clone()),
			None => Err(OsError::INVALID_ARGUMENT),
		}
	}
}

// NodeMap }}}

// HandlesMap {{{

struct HandlesMap {
	next_handle_id: u64,
	open_dirs: HashMap<u64, Arc<dyn DirectoryHandle>>,
	open_files: HashMap<u64, Arc<dyn FileHandle>>,
}

impl HandlesMap {
	fn new() -> HandlesMap {
		Self {
			next_handle_id: 1,
			open_dirs: HashMap::new(),
			open_files: HashMap::new(),
		}
	}

	fn open_file(&mut self, open_file: Arc<dyn FileHandle>) -> u64 {
		let handle_id = self.next_handle_id;
		self.next_handle_id += 1;
		self.open_files.insert(handle_id, open_file);
		handle_id
	}

	fn open_dir(&mut self, open_dir: Arc<dyn DirectoryHandle>) -> u64 {
		let handle_id = self.next_handle_id;
		self.next_handle_id += 1;
		self.open_dirs.insert(handle_id, open_dir);
		handle_id
	}

	fn get_file(
		&self,
		_node_id: NodeId,
		handle: u64,
	) -> Result<&Arc<dyn FileHandle>, Error> {
		match self.open_files.get(&handle) {
			Some(entry) => Ok(entry),
			None => Err(OsError::INVALID_ARGUMENT),
		}
	}

	fn get_dir(
		&self,
		_node_id: NodeId,
		handle: u64,
	) -> Result<&Arc<dyn DirectoryHandle>, Error> {
		match self.open_dirs.get(&handle) {
			Some(entry) => Ok(entry),
			None => Err(OsError::INVALID_ARGUMENT),
		}
	}

	fn close_file(&mut self, _node_id: NodeId, handle: u64) {
		use std::collections::hash_map::Entry;
		if let Entry::Occupied(entry) = self.open_files.entry(handle) {
			entry.remove();
		}
	}

	fn close_dir(&mut self, _node_id: NodeId, handle: u64) {
		use std::collections::hash_map::Entry;
		if let Entry::Occupied(entry) = self.open_dirs.entry(handle) {
			entry.remove();
		}
	}
}

// HandlesMap }}}

// StaticDirectoryHandle {{{

pub struct StaticDirectoryHandle {
	entries: Vec<StaticDirectoryEntry>,
}

impl StaticDirectoryHandle {
	#[must_use]
	pub fn new(entries: Vec<StaticDirectoryEntry>) -> StaticDirectoryHandle {
		Self { entries }
	}
}

pub struct StaticDirectoryEntry {
	name: &'static NodeName,
	pub node_attr: fuse::NodeAttr,
	pub generation: u64,
	pub entry_cache_timeout: Duration,
	pub attr_cache_timeout: Duration,
}

impl StaticDirectoryEntry {
	#[must_use]
	pub fn new(
		name: &'static NodeName,
		node_id: NodeId,
	) -> StaticDirectoryEntry {
		Self {
			name,
			node_attr: fuse::NodeAttr::new(node_id),
			generation: 0,
			entry_cache_timeout: Duration::ZERO,
			attr_cache_timeout: Duration::ZERO,
		}
	}
}

impl DirectoryHandle for StaticDirectoryHandle {
	fn as_readdir_handle(&self) -> Option<&dyn ReaddirHandle> {
		Some(self)
	}

	fn as_readdirplus_handle(&self) -> Option<&dyn ReaddirplusHandle> {
		Some(self)
	}
}

impl ReaddirHandle for StaticDirectoryHandle {
	fn readdir(
		&self,
		_header: &RequestHeader,
		request: server::ReaddirRequest<'_>,
	) -> Result<ReaddirResult, Error> {
		use fuse::server::ReaddirEntry as Dirent;

		let start_offset = request.offset().map_or(0, |o| o.get());
		if start_offset >= self.entries.len() as u64 {
			return Ok(ReaddirResult::new(b""));
		}

		let size = cmp::min(request.size(), u32::from(u16::MAX));
		let mut buf = vec![0u8; size as usize];
		let mut writer = server::ReaddirEntriesWriter::new(&mut buf);

		let mut offset = NonZeroU64::MIN.saturating_add(start_offset);
		for entry in &self.entries[start_offset as usize..] {
			let node_id = entry.node_attr.node_id();
			let mut dirent = Dirent::new(node_id, entry.name, offset);
			offset = offset.saturating_add(1);
			let file_mode = entry.node_attr.mode();
			if let Some(file_type) = fuse::FileType::from_mode(file_mode) {
				dirent.set_file_type(file_type);
			}
			if writer.try_push(&dirent).is_err() {
				break;
			}
		}

		let buf_len = writer.position();
		buf.truncate(buf_len);
		Ok(ReaddirResult::new(buf))
	}
}

impl ReaddirplusHandle for StaticDirectoryHandle {
	fn readdirplus(
		&self,
		_header: &RequestHeader,
		request: server::ReaddirplusRequest<'_>,
	) -> Result<ReaddirplusResult, Error> {
		use fuse::server::ReaddirplusEntry as Dirent;

		let start_offset = request.offset().map_or(0, |o| o.get());
		if start_offset >= self.entries.len() as u64 {
			return Ok(ReaddirplusResult::new(b""));
		}

		let size = cmp::min(request.size(), u32::from(u16::MAX));
		let mut buf = vec![0u8; size as usize];
		let mut writer = server::ReaddirplusEntriesWriter::new(&mut buf);

		let mut offset = NonZeroU64::MIN.saturating_add(start_offset);
		for entry in &self.entries[start_offset as usize..] {
			let mut out_entry = fuse::Entry::new(entry.node_attr);
			out_entry.set_generation(entry.generation);
			out_entry.set_cache_timeout(entry.entry_cache_timeout);
			out_entry.set_attribute_cache_timeout(entry.attr_cache_timeout);
			let dirent = Dirent::new(entry.name, offset, out_entry);
			offset = offset.saturating_add(1);
			if writer.try_push(&dirent).is_err() {
				break;
			}
		}

		let buf_len = writer.position();
		buf.truncate(buf_len);
		Ok(ReaddirplusResult::new(buf))
	}
}

// StaticDirectoryHandle }}}

fn split_duration(d: Duration, out_sec: &mut u64, out_nsec: &mut u32) {
	if d.is_zero() {
		return;
	}
	let (seconds, nanos) = match i64::try_from(d.as_secs()) {
		Ok(seconds) => (seconds, d.subsec_nanos()),
		Err(_) => (i64::MAX, 0),
	};
	*out_sec = seconds as u64;
	*out_nsec = nanos;
}
