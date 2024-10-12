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

//! OS-specific functionality.

#[cfg(any(doc, target_os = "freebsd"))]
pub mod freebsd;

#[cfg(any(doc, target_os = "linux"))]
pub mod linux;

/// The maximum length of a node name, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on node names.
///
/// | Platform | Symbolic constant | Value |
/// |----------|-------------------|-------|
/// | FreeBSD  | `NAME_MAX`        | 255   |
/// | Linux    | `FUSE_NAME_MAX`   | 1024  |
pub const NAME_MAX: Option<usize> = name_max();

const fn name_max() -> Option<usize> {
	if cfg!(target_os = "freebsd") {
		Some(255) // NAME_MAX
	} else if cfg!(target_os = "linux") {
		Some(1024) // FUSE_NAME_MAX
	} else {
		None
	}
}

/// The maximum length of an extended attribute name list, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on extended attribute names.
///
/// | Platform | Symbolic constant in `limits.h` | XattrValue       |
/// |----------|---------------------------------|-------------|
/// | FreeBSD  |                                 | (unlimited) |
/// | Linux    | `XATTR_LIST_MAX`                | 65536       |
///
/// Note that even if the platform imposes no limit, there is still an implicit
/// limit of approximately [`u32::MAX`] implied by the FUSE protocol.
pub const XATTR_LIST_MAX: Option<usize> = xattr_list_max();

const fn xattr_list_max() -> Option<usize> {
	if cfg!(target_os = "linux") {
		Some(65536) // XATTR_LIST_MAX
	} else {
		None
	}
}

/// The maximum length of an extended attribute name, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on extended attribute names.
///
/// | Platform | Symbolic constant in `limits.h` | Value |
/// |----------|---------------------------------|-------|
/// | FreeBSD  | `EXTATTR_MAXNAMELEN`            | 255   |
/// | Linux    | `XATTR_NAME_MAX`                | 255   |
pub const XATTR_NAME_MAX: Option<usize> = xattr_name_max();

const fn xattr_name_max() -> Option<usize> {
	if cfg!(target_os = "freebsd") {
		Some(255) // EXTATTR_MAXNAMELEN
	} else if cfg!(target_os = "linux") {
		Some(255) // XATTR_NAME_MAX
	} else {
		None
	}
}

/// The maximum length of an extended attribute value, in bytes.
///
/// This value is platform-specific. If `None`, then the platform does not
/// impose a maximum length on extended attribute names.
///
/// | Platform | Symbolic constant in `limits.h` | XattrValue       |
/// |----------|---------------------------------|-------------|
/// | FreeBSD  |                                 | (unlimited) |
/// | Linux    | `XATTR_SIZE_MAX`                | 65536       |
///
/// Note that even if the platform imposes no limit on the maximum length
/// of an extended attribute value, there is still an implicit limit of
/// approximately [`u32::MAX`] implied by the FUSE protocol.
pub const XATTR_SIZE_MAX: Option<usize> = xattr_size_max();

const fn xattr_size_max() -> Option<usize> {
	if cfg!(target_os = "linux") {
		Some(65536) // XATTR_SIZE_MAX
	} else {
		None
	}
}
