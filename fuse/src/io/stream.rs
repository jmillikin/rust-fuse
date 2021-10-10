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

use core::future::Future;
use core::num::NonZeroUsize;

pub trait InputStream {
	type Error;

	fn recv(&self, buf: &mut [u8]) -> Result<Option<NonZeroUsize>, Self::Error>;
}

pub trait AsyncInputStream {
	type Error;
	type Future: Future<Output = Result<Option<NonZeroUsize>, Self::Error>>;

	fn recv(&self, buf: &mut [u8]) -> Self::Future;
}

pub trait OutputStream {
	type Error;

	fn send(&self, buf: &[u8]) -> Result<(), Self::Error>;

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Result<(), Self::Error>;
}

pub trait AsyncOutputStream {
	type Error;
	type Future: Future<Output = Result<(), Self::Error>>;

	fn send(&self, buf: &[u8]) -> Self::Future;

	fn send_vectored<const N: usize>(
		&self,
		bufs: &[&[u8]; N],
	) -> Self::Future;
}
