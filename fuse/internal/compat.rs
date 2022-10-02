// Copyright 2022 John Millikin and the rust-fuse contributors.
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

use crate::internal::fuse_kernel;

#[derive(Clone, Copy)]
pub(crate) struct Versioned<U> {
	version_minor: u32,
	u: U,
}

// fuse_create_in {{{

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_create_in<'a> {
	v7p1: &'a fuse_create_in_v7p1,
	v7p12: &'a fuse_kernel::fuse_create_in,
}

#[repr(C)]
pub(crate) struct fuse_create_in_v7p1 {
	pub(crate) flags: u32,
	pub(crate) unused: u32,
}

impl<'a> Versioned<fuse_create_in<'a>> {
	#[inline]
	pub(crate) fn new_create_v7p1(
		version_minor: u32,
		v7p1: &'a fuse_create_in_v7p1,
	) -> Self {
		Self {
			version_minor,
			u: fuse_create_in { v7p1 },
		}
	}

	#[inline]
	pub(crate) fn new_create_v7p12(
		version_minor: u32,
		v7p12: &'a fuse_kernel::fuse_create_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_create_in { v7p12 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p1(self) -> &'a fuse_create_in_v7p1 {
		unsafe { self.u.v7p1 }
	}

	#[inline]
	pub(crate) fn as_v7p12(self) -> Option<&'a fuse_kernel::fuse_create_in> {
		if self.version_minor >= 12 {
			return Some(unsafe { self.u.v7p12 });
		}
		None
	}
}

// }}}

// fuse_getattr_in {{{

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_getattr_in<'a> {
	v7p1: &'a (),
	v7p9: &'a fuse_kernel::fuse_getattr_in,
}

impl<'a> Versioned<fuse_getattr_in<'a>> {
	#[inline]
	pub(crate) fn new_getattr_v7p1(
		version_minor: u32,
	) -> Self {
		Self {
			version_minor,
			u: fuse_getattr_in { v7p1: &() },
		}
	}

	#[inline]
	pub(crate) fn new_getattr_v7p9(
		version_minor: u32,
		v7p9: &'a fuse_kernel::fuse_getattr_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_getattr_in { v7p9 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p9(self) -> Option<&'a fuse_kernel::fuse_getattr_in> {
		if self.version_minor >= 9 {
			return Some(unsafe { self.u.v7p9 });
		}
		None
	}
}

// }}}

// fuse_mknod_in {{{

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_mknod_in<'a> {
	v7p1: &'a fuse_mknod_in_v7p1,
	v7p12: &'a fuse_kernel::fuse_mknod_in,
}

#[repr(C)]
pub(crate) struct fuse_mknod_in_v7p1 {
	pub(crate) mode: u32,
	pub(crate) rdev: u32,
}

impl<'a> Versioned<fuse_mknod_in<'a>> {
	#[inline]
	pub(crate) fn new_mknod_v7p1(
		version_minor: u32,
		v7p1: &'a fuse_mknod_in_v7p1,
	) -> Self {
		Self {
			version_minor,
			u: fuse_mknod_in { v7p1 },
		}
	}

	#[inline]
	pub(crate) fn new_mknod_v7p12(
		version_minor: u32,
		v7p12: &'a fuse_kernel::fuse_mknod_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_mknod_in { v7p12 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p1(self) -> &'a fuse_mknod_in_v7p1 {
		unsafe { self.u.v7p1 }
	}

	#[inline]
	pub(crate) fn as_v7p12(self) -> Option<&'a fuse_kernel::fuse_mknod_in> {
		if self.version_minor >= 12 {
			return Some(unsafe { self.u.v7p12 });
		}
		None
	}
}

// }}}

// fuse_read_in {{{

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_read_in<'a> {
	v7p1: &'a fuse_read_in_v7p1,
	v7p9: &'a fuse_kernel::fuse_read_in,
}

#[repr(C)]
pub(crate) struct fuse_read_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) offset: u64,
	pub(crate) size: u32,
	pub(crate) padding: u32,
}

impl<'a> Versioned<fuse_read_in<'a>> {
	#[inline]
	pub(crate) fn new_read_v7p1(
		version_minor: u32,
		v7p1: &'a fuse_read_in_v7p1,
	) -> Self {
		Self {
			version_minor,
			u: fuse_read_in { v7p1 },
		}
	}

	#[inline]
	pub(crate) fn new_read_v7p9(
		version_minor: u32,
		v7p9: &'a fuse_kernel::fuse_read_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_read_in { v7p9 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p1(self) -> &'a fuse_read_in_v7p1 {
		unsafe { self.u.v7p1 }
	}

	#[inline]
	pub(crate) fn as_v7p9(self) -> Option<&'a fuse_kernel::fuse_read_in> {
		if self.version_minor >= 9 {
			return Some(unsafe { self.u.v7p9 });
		}
		None
	}
}

// }}}

/// fuse_release_in {{{

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_release_in<'a> {
	v7p1: &'a fuse_release_in_v7p1,
	v7p8: &'a fuse_kernel::fuse_release_in,
}

#[repr(C)]
pub(crate) struct fuse_release_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) flags: u32,
	pub(crate) padding: u32,
}

impl<'a> Versioned<fuse_release_in<'a>> {
	#[inline]
	pub(crate) fn new_release_v7p1(
		version_minor: u32,
		v7p1: &'a fuse_release_in_v7p1,
	) -> Self {
		Self {
			version_minor,
			u: fuse_release_in { v7p1 },
		}
	}

	#[inline]
	pub(crate) fn new_release_v7p8(
		version_minor: u32,
		v7p8: &'a fuse_kernel::fuse_release_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_release_in { v7p8 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p1(self) -> &'a fuse_release_in_v7p1 {
		unsafe { self.u.v7p1 }
	}

	#[inline]
	pub(crate) fn as_v7p8(self) -> Option<&'a fuse_kernel::fuse_release_in> {
		if self.version_minor >= 8 {
			return Some(unsafe { self.u.v7p8 });
		}
		None
	}
}

// }}}

// fuse_write_in {{{

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union fuse_write_in<'a> {
	v7p1: &'a fuse_write_in_v7p1,
	v7p9: &'a fuse_kernel::fuse_write_in,
}

#[repr(C)]
pub(crate) struct fuse_write_in_v7p1 {
	pub(crate) fh: u64,
	pub(crate) offset: u64,
	pub(crate) size: u32,
	pub(crate) write_flags: u32,
}

impl<'a> Versioned<fuse_write_in<'a>> {
	#[inline]
	pub(crate) fn new_write_v7p1(
		version_minor: u32,
		v7p1: &'a fuse_write_in_v7p1,
	) -> Self {
		Self {
			version_minor,
			u: fuse_write_in { v7p1 },
		}
	}

	#[inline]
	pub(crate) fn new_write_v7p9(
		version_minor: u32,
		v7p9: &'a fuse_kernel::fuse_write_in,
	) -> Self {
		Self {
			version_minor,
			u: fuse_write_in { v7p9 },
		}
	}

	#[inline]
	pub(crate) fn as_v7p1(self) -> &'a fuse_write_in_v7p1 {
		unsafe { self.u.v7p1 }
	}

	#[inline]
	pub(crate) fn as_v7p9(self) -> Option<&'a fuse_kernel::fuse_write_in> {
		if self.version_minor >= 9 {
			return Some(unsafe { self.u.v7p9 });
		}
		None
	}
}


// }}}
