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

/// Types and constants defined by the FUSE kernel interface.
///
/// This module is automatically generated from [`fuse.h`] in the Linux kernel
/// source tree.
///
/// [`fuse.h`]: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/linux/fuse.h?h=v5.19
#[allow(
	dead_code,
	missing_docs,
	non_camel_case_types,
	unused_parens,
)]
pub mod fuse_kernel;

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
