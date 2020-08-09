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

use core::num::NonZeroU16;

macro_rules! error_numbers {
	([ $( $name:ident , )* ]) => {
		$(
			pub(crate) const $name: NonZeroU16 = unsafe {
				NonZeroU16::new_unchecked(target::$name)
			};
		)*
	}
}

macro_rules! error_numbers_target {
	($( $name:ident = $value:literal ; )*) => {
		mod target {
			$(
				pub(super) const $name: u16 = $value;
			)*
		}
	}
}

#[rustfmt::skip]
error_numbers!([
	ENODEV,
	ENOENT,
	ENOSYS,
	ERANGE,
]);

#[cfg(all(
	target_os = "linux",
	any(target_arch = "x86", target_arch = "x86_64",),
))]
error_numbers_target! {
	ENODEV = 19;
	ENOENT = 2;
	ENOSYS = 38;
	ERANGE = 34;
}
