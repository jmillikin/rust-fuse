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

#[macro_use]
mod bitflags;

pub(crate) mod compat;
pub(crate) mod debug;
pub(crate) mod dirent;

pub(crate) mod timestamp;

macro_rules! new {
	($t:ty { $( $field:ident : $value:expr , )+ }) => {{
		let mut value = <$t>::new();
		$(
			value.$field = $value;
		)+
		value
	}}
}

macro_rules! try_from_cuse_request {
	($t:ty, |$request:ident| $try_from:tt) => {
		impl<'a> TryFrom<crate::server::CuseRequest<'a>> for $t {
			type Error = crate::server::RequestError;

			fn try_from(
				$request: crate::server::CuseRequest<'a>,
			) -> Result<Self, crate::server::RequestError> {
				$try_from
			}
		}
	}
}

macro_rules! try_from_fuse_request {
	($t:ty, |$request:ident| $try_from:tt) => {
		impl<'a> TryFrom<crate::server::FuseRequest<'a>> for $t {
			type Error = crate::server::RequestError;

			fn try_from(
				$request: crate::server::FuseRequest<'a>,
			) -> Result<Self, crate::server::RequestError> {
				$try_from
			}
		}
	}
}
