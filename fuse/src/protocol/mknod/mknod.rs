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

use crate::protocol::prelude::*;

#[cfg(rust_fuse_test = "mknod_test")]
mod mknod_test;

// MknodRequest {{{

/// Request type for [`FuseHandlers::mknod`].
///
/// [`FuseHandlers::mknod`]: ../../trait.FuseHandlers.html#method.mknod
pub struct MknodRequest<'a> {
	parent_id: NodeId,
	name: &'a NodeName,
	raw: fuse_kernel::fuse_mknod_in,
}

impl MknodRequest<'_> {
	pub fn parent_id(&self) -> NodeId {
		self.parent_id
	}

	pub fn name(&self) -> &NodeName {
		self.name
	}

	pub fn mode(&self) -> FileMode {
		FileMode(self.raw.mode)
	}

	pub fn umask(&self) -> u32 {
		self.raw.umask
	}

	pub fn device_number(&self) -> Option<u32> {
		match self.mode().file_type() {
			Some(FileType::CharDevice) | Some(FileType::BlockDevice) => {
				Some(self.raw.rdev)
			},
			_ => None,
		}
	}
}

impl fmt::Debug for MknodRequest<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MknodRequest")
			.field("parent_id", &self.parent_id())
			.field("name", &self.name())
			.field("mode", &self.mode())
			.field("umask", &format_args!("{:#o}", &self.raw.umask))
			.field("device_number", &format_args!("{:?}", self.device_number()))
			.finish()
	}
}

#[repr(C)]
pub(crate) struct fuse_mknod_in_v7p1 {
	pub mode: u32,
	pub rdev: u32,
}

impl<'a> fuse_io::DecodeRequest<'a> for MknodRequest<'a> {
	fn decode_request(
		mut dec: fuse_io::RequestDecoder<'a>,
	) -> Result<Self, Error> {
		let header = dec.header();
		debug_assert!(header.opcode == fuse_kernel::FUSE_MKNOD);

		let parent_id = try_node_id(header.nodeid)?;

		if dec.version().minor() < 12 {
			let raw: &fuse_mknod_in_v7p1 = dec.next_sized()?;
			let name = NodeName::new(dec.next_nul_terminated_bytes()?);
			return Ok(Self {
				parent_id,
				name,
				raw: fuse_kernel::fuse_mknod_in {
					mode: raw.mode,
					rdev: raw.rdev,
					umask: 0,
					padding: 0,
				},
			});
		}

		let raw: &fuse_kernel::fuse_mknod_in = dec.next_sized()?;
		let name = NodeName::new(dec.next_nul_terminated_bytes()?);
		Ok(Self {
			parent_id,
			name,
			raw: *raw,
		})
	}
}

// }}}

// MknodResponse {{{

/// Response type for [`FuseHandlers::mknod`].
///
/// [`FuseHandlers::mknod`]: ../../trait.FuseHandlers.html#method.mknod
pub struct MknodResponse<'a> {
	phantom: PhantomData<&'a ()>,
	raw: fuse_kernel::fuse_entry_out,
}

impl<'a> MknodResponse<'a> {
	pub fn new() -> MknodResponse<'a> {
		Self {
			phantom: PhantomData,
			raw: Default::default(),
		}
	}

	pub fn node(&self) -> &Node {
		Node::new_ref(&self.raw)
	}

	pub fn node_mut(&mut self) -> &mut Node {
		Node::new_ref_mut(&mut self.raw)
	}
}

impl fmt::Debug for MknodResponse<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MknodResponse")
			.field("node", &self.node())
			.finish()
	}
}

impl fuse_io::EncodeResponse for MknodResponse<'_> {
	fn encode_response<'a, S: io::OutputStream>(
		&'a self,
		enc: fuse_io::ResponseEncoder<S>,
	) -> Result<(), S::Error> {
		self.node().encode_entry(enc)
	}
}

// }}}
