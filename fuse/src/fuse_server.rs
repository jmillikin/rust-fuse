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

use std::io;
use std::sync::{Arc, Mutex};

use crate::fuse_handlers::FuseHandlers;
use crate::internal::errors;
use crate::internal::fuse_io::{
	self,
	AlignedBuffer,
	Channel,
	DecodeRequest,
	EncodeResponse,
};
use crate::internal::fuse_kernel;
use crate::protocol;
use crate::server;

#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
pub struct FuseServer<Handlers, Mount> {
	fuse_device: fuse_io::FileChannel,
	mount: Mount,
	handlers: Arc<Handlers>,
	executor: Arc<Mutex<FuseServerExecutor<Handlers>>>,
	fuse_version: crate::ProtocolVersion,
}

impl<Handlers, Mount> FuseServer<Handlers, Mount>
where
	Handlers: FuseHandlers,
	Mount: FuseMount,
{
	fn new(
		fuse_device: fuse_io::FileChannel,
		mount: Mount,
		mut handlers: Handlers,
	) -> std::io::Result<Self> {
		let executor_channel = fuse_device.try_clone()?;
		let init_response = fuse_handshake(&fuse_device, &mut handlers)?;
		let fuse_version = init_response.version();
		let handlers_arc = Arc::new(handlers);
		let executor_handlers = handlers_arc.clone();
		Ok(Self {
			fuse_device,
			mount,
			handlers: handlers_arc,
			executor: Arc::new(Mutex::new(FuseServerExecutor::new(
				executor_channel,
				executor_handlers,
				1048576, /* 1 MiB; TODO read from init_response */
				fuse_version,
			))),
			fuse_version,
		})
	}
}

impl<Handlers, Mount> FuseServer<Handlers, Mount>
where
	Mount: FuseMount,
{
	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn unmount(self) -> io::Result<()> {
		self.mount.unmount()
	}
}

impl<Handlers, Mount> FuseServer<Handlers, Mount> {
	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn executor(&self) -> &Arc<Mutex<FuseServerExecutor<Handlers>>> {
		&self.executor
	}

	#[doc(hidden)]
	pub fn new_executor(&self) -> io::Result<FuseServerExecutor<Handlers>> {
		let _ = self.fuse_device;
		let _ = self.handlers;
		let _ = self.fuse_version;
		todo!("FuseServer::new_executor")
	}
}

fn fuse_handshake<Channel, Handlers>(
	channel: &Channel,
	handlers: &mut Handlers,
) -> io::Result<protocol::FuseInitResponse>
where
	Channel: fuse_io::Channel,
	Handlers: FuseHandlers,
{
	let mut read_buf = fuse_io::MinReadBuffer::new();

	loop {
		let request_size = channel.read(read_buf.get_mut())?;
		let request_buf = fuse_io::aligned_slice(&read_buf, request_size);
		let request_decoder = fuse_io::RequestDecoder::new(
			request_buf,
			crate::ProtocolVersion::LATEST,
		)?;

		let request_header = request_decoder.header();
		if request_header.opcode != fuse_kernel::FUSE_INIT {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				format!(
					"Received bad opcode {:?} from kernel (expected FUSE_INIT)",
					request_header.opcode
				),
			));
		}

		let request_id = request_header.unique;
		let init_request =
			protocol::FuseInitRequest::decode_request(request_decoder)?;

		let major_version = init_request.version().major();
		if major_version != fuse_kernel::FUSE_KERNEL_VERSION {
			let init_response =
				protocol::FuseInitResponse::new(crate::ProtocolVersion::LATEST);
			init_response.encode_response(fuse_io::ResponseEncoder::new(
				channel,
				request_id,
				init_response.version(),
			))?;
			continue;
		}

		let init_response = handlers.fuse_init(&init_request);
		init_response.encode_response(fuse_io::ResponseEncoder::new(
			channel,
			request_id,
			// FuseInitResponse always encodes with its own version
			crate::ProtocolVersion::LATEST,
		))?;
		return Ok(init_response);
	}
}

#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
pub trait FuseMountOptions {
	type Mount: FuseMount;
}

#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
pub trait FuseMount: Sized {
	type Options: FuseMountOptions;

	#[doc(hidden)]
	fn mount(
		mount_target: &std::path::Path,
		options: Option<Self::Options>,
	) -> io::Result<(std::fs::File, Self)>;

	#[doc(hidden)]
	fn unmount(self) -> io::Result<()>;
}

#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
pub struct FuseServerBuilder<Handlers, MountOptions> {
	handlers: Handlers,
	mount_options: Option<MountOptions>,
}

impl<Handlers, MountOptions> FuseServerBuilder<Handlers, MountOptions>
where
	Handlers: FuseHandlers,
	MountOptions: FuseMountOptions,
{
	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn new(handlers: Handlers) -> Self {
		Self {
			mount_options: None,
			handlers,
		}
	}

	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn set_mount_options(mut self, mount_options: MountOptions) -> Self {
		self.mount_options = Some(mount_options);
		self
	}

	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn mount<Mount, Path>(
		self,
		mount_target: Path,
	) -> io::Result<FuseServer<Handlers, Mount>>
	where
		Path: AsRef<std::path::Path>,
		Mount: FuseMount<Options = MountOptions>,
		MountOptions: FuseMountOptions<Mount = Mount>,
	{
		let (fuse_device, mount) = <MountOptions::Mount as FuseMount>::mount(
			mount_target.as_ref(),
			self.mount_options,
		)?;

		FuseServer::new(
			fuse_io::FileChannel::new(fuse_device),
			mount,
			self.handlers,
		)
	}
}

#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
pub struct FuseServerExecutor<Handlers> {
	channel: Arc<fuse_io::FileChannel>,
	handlers: Arc<Handlers>,
	read_buf: fuse_io::AlignedVec,
	fuse_version: crate::ProtocolVersion,
}

impl<Handlers: FuseHandlers> FuseServerExecutor<Handlers> {
	fn new(
		channel: fuse_io::FileChannel,
		handlers: Arc<Handlers>,
		read_buf_size: usize,
		fuse_version: crate::ProtocolVersion,
	) -> Self {
		Self {
			channel: Arc::new(channel),
			handlers,
			read_buf: fuse_io::AlignedVec::new(read_buf_size),
			fuse_version,
		}
	}

	#[cfg_attr(doc, doc(cfg(feature = "unstable")))]
	pub fn run(&mut self) -> io::Result<()> {
		let handlers = &*self.handlers;
		loop {
			let request_size = match self.channel.read(self.read_buf.get_mut())
			{
				Err(err) => {
					if err.raw_os_error() == Some(errors::ENODEV.get() as i32) {
						return Ok(());
					} else {
						return Err(err);
					}
				},
				Ok(request_size) => request_size,
			};
			let request_buf =
				fuse_io::aligned_slice(&self.read_buf, request_size);
			let decoder =
				fuse_io::RequestDecoder::new(request_buf, self.fuse_version)?;

			fuse_request_dispatch(handlers, decoder, &self.channel)?;
		}
	}
}

fn fuse_request_dispatch<Handlers: FuseHandlers>(
	handlers: &Handlers,
	request_decoder: fuse_io::RequestDecoder,
	channel: &Arc<fuse_io::FileChannel>,
) -> std::io::Result<()> {
	let header = request_decoder.header();

	let fuse_version = request_decoder.version();
	let ctx = server::ServerContext::new(*header);

	macro_rules! do_dispatch {
		($handler:tt) => {{
			let request = DecodeRequest::decode_request(request_decoder)?;
			let respond_once = server::RespondOnceImpl::new(
				channel,
				header.unique,
				fuse_version,
			);
			handlers.$handler(ctx, &request, respond_once);
		}};
	}

	match header.opcode {
		#[cfg(feature = "unstable_fuse_access")]
		fuse_kernel::FUSE_ACCESS => do_dispatch!(access),
		#[cfg(feature = "unstable_fuse_bmap")]
		fuse_kernel::FUSE_BMAP => do_dispatch!(bmap),
		#[cfg(feature = "unstable_fuse_create")]
		fuse_kernel::FUSE_CREATE => do_dispatch!(create),
		#[cfg(feature = "unstable_fuse_fallocate")]
		fuse_kernel::FUSE_FALLOCATE => do_dispatch!(fallocate),
		#[cfg(feature = "unstable_fuse_flush")]
		fuse_kernel::FUSE_FLUSH => do_dispatch!(flush),
		fuse_kernel::FUSE_FORGET | fuse_kernel::FUSE_BATCH_FORGET => {
			let request = DecodeRequest::decode_request(request_decoder)?;
			handlers.forget(ctx, &request);
		},
		#[cfg(feature = "unstable_fuse_fsync")]
		fuse_kernel::FUSE_FSYNC => do_dispatch!(fsync),
		#[cfg(feature = "unstable_fuse_fsyncdir")]
		fuse_kernel::FUSE_FSYNCDIR => do_dispatch!(fsyncdir),
		fuse_kernel::FUSE_GETATTR => do_dispatch!(getattr),
		#[cfg(feature = "unstable_fuse_getlk")]
		fuse_kernel::FUSE_GETLK => do_dispatch!(getlk),
		fuse_kernel::FUSE_GETXATTR => do_dispatch!(getxattr),
		#[cfg(feature = "unstable_fuse_ioctl")]
		fuse_kernel::FUSE_IOCTL => do_dispatch!(ioctl),
		#[cfg(feature = "unstable_fuse_link")]
		fuse_kernel::FUSE_LINK => do_dispatch!(link),
		fuse_kernel::FUSE_LISTXATTR => do_dispatch!(listxattr),
		fuse_kernel::FUSE_LOOKUP => do_dispatch!(lookup),
		#[cfg(feature = "unstable_fuse_lseek")]
		fuse_kernel::FUSE_LSEEK => do_dispatch!(lseek),
		#[cfg(feature = "unstable_fuse_mkdir")]
		fuse_kernel::FUSE_MKDIR => do_dispatch!(mkdir),
		#[cfg(feature = "unstable_fuse_mknod")]
		fuse_kernel::FUSE_MKNOD => do_dispatch!(mknod),
		fuse_kernel::FUSE_OPEN => do_dispatch!(open),
		fuse_kernel::FUSE_OPENDIR => do_dispatch!(opendir),
		fuse_kernel::FUSE_READ => do_dispatch!(read),
		fuse_kernel::FUSE_READDIR => do_dispatch!(readdir),
		fuse_kernel::FUSE_READLINK => do_dispatch!(readlink),
		fuse_kernel::FUSE_RELEASE => do_dispatch!(release),
		fuse_kernel::FUSE_RELEASEDIR => do_dispatch!(releasedir),
		#[cfg(feature = "unstable_fuse_removexattr")]
		fuse_kernel::FUSE_REMOVEXATTR => do_dispatch!(removexattr),
		#[cfg(feature = "unstable_fuse_rename")]
		fuse_kernel::FUSE_RENAME => do_dispatch!(rename),
		#[cfg(feature = "unstable_fuse_rename")]
		fuse_kernel::FUSE_RENAME2 => do_dispatch!(rename),
		#[cfg(feature = "unstable_fuse_rmdir")]
		fuse_kernel::FUSE_RMDIR => do_dispatch!(rmdir),
		#[cfg(feature = "unstable_fuse_setattr")]
		fuse_kernel::FUSE_SETATTR => do_dispatch!(setattr),
		#[cfg(feature = "unstable_fuse_setlk")]
		fuse_kernel::FUSE_SETLK => do_dispatch!(setlk),
		#[cfg(feature = "unstable_fuse_setxattr")]
		fuse_kernel::FUSE_SETXATTR => do_dispatch!(setxattr),
		#[cfg(feature = "unstable_fuse_statfs")]
		fuse_kernel::FUSE_STATFS => do_dispatch!(statfs),
		#[cfg(feature = "unstable_fuse_symlink")]
		fuse_kernel::FUSE_SYMLINK => do_dispatch!(symlink),
		#[cfg(feature = "unstable_fuse_unlink")]
		fuse_kernel::FUSE_UNLINK => do_dispatch!(unlink),
		#[cfg(feature = "unstable_fuse_write")]
		fuse_kernel::FUSE_WRITE => do_dispatch!(write),
		_ => {
			let request =
				protocol::UnknownRequest::decode_request(request_decoder)?;
			let respond_once = server::RespondOnceImpl::new(
				channel,
				header.unique,
				fuse_version,
			);
			// handlers.unknown(ctx, &request);
			// TODO: use ServerLogger to log the unknown request
			let _ = request;
			respond_once.encoder().encode_error(errors::ENOSYS);
		},
	}
	Ok(())
}
