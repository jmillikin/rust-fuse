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

use crate::MIN_READ_BUFFER;

pub struct ArrayBuffer(ArrayBufferImpl);

#[repr(align(8))]
struct ArrayBufferImpl([u8; MIN_READ_BUFFER]);

impl ArrayBuffer {
	pub fn new() -> Self {
		ArrayBuffer(ArrayBufferImpl([0u8; MIN_READ_BUFFER]))
	}

	pub fn borrow(&self) -> &[u8] {
		&self.0 .0
	}
	pub fn borrow_mut(&mut self) -> &mut [u8] {
		&mut self.0 .0
	}
}
