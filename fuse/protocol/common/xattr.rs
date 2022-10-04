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

use core::fmt;

// XattrError {{{

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct XattrError {
	kind: XattrErrorKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum XattrErrorKind {
	ExceedsListMax(usize),
	ExceedsCapacity(usize, usize),
	ExceedsSizeMax(usize),
	ExceedsRequestSize(usize, u32),
}

impl XattrError {
	pub(crate) fn exceeds_list_max(response_size: usize) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsListMax(response_size),
		}
	}

	pub(crate) fn exceeds_capacity(
		response_size: usize,
		capacity: usize,
	) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsCapacity(response_size, capacity),
		}
	}

	pub(crate) fn exceeds_size_max(value_size: usize) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsSizeMax(value_size),
		}
	}

	pub(crate) fn exceeds_request_size(
		response_size: usize,
		request_size: u32,
	) -> XattrError {
		XattrError {
			kind: XattrErrorKind::ExceedsRequestSize(
				response_size,
				request_size,
			),
		}
	}
}

impl fmt::Display for XattrError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		fmt::Debug::fmt(self, fmt)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for XattrError {}

// }}}
