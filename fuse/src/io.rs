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

mod buffer;
pub(crate) mod decode;
pub(crate) mod encode;
mod error;
mod stream;
mod version;

pub use self::buffer::{ArrayBuffer, Buffer, MIN_READ_BUFFER};

#[cfg(feature = "std")]
pub use self::buffer::PinnedBuffer;

pub use self::error::DecodeError;

pub use self::stream::{
	AsyncInputStream,
	AsyncOutputStream,
	InputStream,
	OutputStream,
};

pub use self::version::ProtocolVersion;

// compatibility
pub use crate::channel::{Channel, ChannelError};
pub use crate::cuse_server::CuseServerChannel;
pub use crate::fuse_server::FuseServerChannel;
pub use crate::old_server::ServerChannel;
