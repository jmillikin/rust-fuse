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

//! Linux-specific functionality.

use core::ffi;
use core::fmt;
use core::fmt::Write;

use crate::node;

const CSTR_FUSE: &ffi::CStr = unsafe {
	ffi::CStr::from_bytes_with_nul_unchecked(b"fuse\0")
};

const CSTR_FUSEBLK: &ffi::CStr = unsafe {
	ffi::CStr::from_bytes_with_nul_unchecked(b"fuseblk\0")
};

const MOUNT_SOURCE_FUSE: &MountSource = unsafe {
	MountSource::new_unchecked(CSTR_FUSE)
};

const MOUNT_TYPE_FUSE: &MountType = unsafe {
	MountType::new_unchecked(CSTR_FUSE)
};

const MOUNT_TYPE_FUSEBLK: &MountType = unsafe {
	MountType::new_unchecked(CSTR_FUSEBLK)
};

// FUSE_DEV_IOC_CLONE {{{

#[cfg(not(any(
	target_arch = "alpha",
	target_arch = "mips",
	target_arch = "mips64",
	target_arch = "parisc",
	target_arch = "powerpc",
	target_arch = "powerpc64",
	target_arch = "sparc",
	target_arch = "sparc64",
)))]
mod arch {
	// _IOC_SIZEBITS == 14 && _IOC_DIRBITS == 2
	pub const FUSE_DEV_IOC_CLONE: u32 = 0x8004E500; // _IOR(229, 0, uint32_t)
}

#[cfg(any(
	target_arch = "alpha",
	target_arch = "mips",
	target_arch = "mips64",
	target_arch = "parisc",
	target_arch = "powerpc",
	target_arch = "powerpc64",
	target_arch = "sparc",
	target_arch = "sparc64",
))]
mod arch {
	// _IOC_SIZEBITS == 13 && _IOC_DIRBITS == 3
	pub const FUSE_DEV_IOC_CLONE: u32 = 0x4004E500; // _IOR(229, 0, uint32_t)
}

/// `ioctl` command for cloning a `/dev/fuse` device handle.
pub const FUSE_DEV_IOC_CLONE: u32 = arch::FUSE_DEV_IOC_CLONE;

// }}}

// MountSource {{{

/// A borrowed FUSE mount source.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MountSource {
	inner: ffi::CStr,
}

impl MountSource {
	/// The mount source `"fuse"`, the default for normal FUSE filesystems.
	pub const FUSE: &'static MountSource = MOUNT_SOURCE_FUSE;

	/// Attempts to reborrow a C string as a mount source.
	///
	/// # Errors
	///
	/// Returns `None` if the C string is empty.
	#[must_use]
	pub fn new(mount_source: &ffi::CStr) -> Option<&MountSource> {
		if cstr_is_empty(mount_source) {
			return None;
		}
		Some(unsafe { Self::new_unchecked(mount_source) })
	}

	/// Reborrows a C string as a mount source, without validation.
	///
	/// # Safety
	///
	/// The provided C string must be non-empty.
	#[must_use]
	pub const unsafe fn new_unchecked(
		mount_source: &ffi::CStr,
	) -> &MountSource {
		&*(mount_source as *const ffi::CStr as *const MountSource)
	}

	/// Returns this mount source as a borrowed C string.
	#[must_use]
	pub const fn as_cstr(&self) -> &ffi::CStr {
		&self.inner
	}
}

impl fmt::Debug for MountSource {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.inner, fmt)
	}
}

// }}}

// MountType {{{

/// A borrowed FUSE mount type.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MountType {
	inner: ffi::CStr,
}

impl MountType {
	/// The mount type `"fuse"`, used for normal FUSE filesystems.
	pub const FUSE: &'static MountType = MOUNT_TYPE_FUSE;

	/// The mount type `"fuseblk"`, used for FUSE-wrapped block devices.
	pub const FUSEBLK: &'static MountType = MOUNT_TYPE_FUSEBLK;

	/// Reborrows a C string as a mount type, without validation.
	///
	/// # Safety
	///
	/// The provided C string must be non-empty.
	#[must_use]
	pub const unsafe fn new_unchecked(mount_type: &ffi::CStr) -> &MountType {
		&*(mount_type as *const ffi::CStr as *const MountType)
	}

	/// Returns this mount type as a borrowed C string.
	#[must_use]
	pub const fn as_cstr(&self) -> &ffi::CStr {
		&self.inner
	}
}

impl fmt::Debug for MountType {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.inner, fmt)
	}
}

// }}}

// FuseSubtype {{{

/// A borrowed FUSE filesystem subtype.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FuseSubtype {
	inner: ffi::CStr,
}

impl FuseSubtype {
	/// Attempts to reborrow a C string as a FUSE filesystem subtype.
	///
	/// # Errors
	///
	/// Returns `None` if the C string is empty or contains a comma.
	#[must_use]
	pub fn new(subtype: &ffi::CStr) -> Option<&FuseSubtype> {
		if cstr_is_empty(subtype) {
			return None;
		}
		if subtype.to_bytes().contains(&b',') {
			return None;
		}
		Some(unsafe { Self::new_unchecked(subtype) })
	}

	/// Reborrows a C string as a FUSE filesystem subtype, without validation.
	///
	/// # Safety
	///
	/// The provided C string must be non-empty and must not contain a comma.
	#[must_use]
	pub const unsafe fn new_unchecked(subtype: &ffi::CStr) -> &FuseSubtype {
		&*(subtype as *const ffi::CStr as *const FuseSubtype)
	}

	/// Returns this FUSE filesystem subtype as a borrowed C string.
	#[must_use]
	pub const fn as_cstr(&self) -> &ffi::CStr {
		&self.inner
	}
}

impl fmt::Debug for FuseSubtype {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.inner, fmt)
	}
}

// }}}

// MountOptions {{{

/// Builder for Linux `mount()` options.
#[derive(Copy, Clone)]
pub struct MountOptions<'a> {
	mount_source: &'a MountSource,
	mount_type: &'a MountType,

	allow_other: bool,
	block_size: Option<u32>,
	default_permissions: bool,
	fuse_device_fd: Option<u32>,
	subtype: Option<&'a FuseSubtype>,
	group_id: Option<u32>,
	max_read: Option<u32>,
	root_mode: Option<node::Mode>,
	user_id: Option<u32>,
}

impl<'a> MountOptions<'a> {
	/// Create a new `MountOptions` with default values.
	#[must_use]
	pub fn new() -> Self {
		MountOptions {
			mount_type: MountType::FUSE,
			mount_source: MountSource::FUSE,

			allow_other: false,
			block_size: None,
			default_permissions: false,
			fuse_device_fd: None,
			subtype: None,
			group_id: None,
			max_read: None,
			root_mode: None,
			user_id: None,
		}
	}

	/// Returns the `allow_other=` mount data value.
	#[must_use]
	pub fn allow_other(&self) -> bool {
		self.allow_other
	}

	/// Sets the `allow_other=` mount data value.
	///
	/// If `allow_other` is `true`, then all users (including root) may access
	/// the filesystem.
	pub fn set_allow_other(&mut self, allow_other: bool) {
		self.allow_other = allow_other;
	}

	/// Returns the `blksize=` mount data value.
	#[must_use]
	pub fn block_size(&self) -> Option<u32> {
		self.block_size
	}

	/// Sets the `blksize=` mount data value.
	///
	/// This option is only valid for the [`FUSEBLK`] mount type. If `None`,
	/// the kernel default block size of 512 bytes will be used.
	///
	/// [`FUSEBLK`]: MountType::FUSEBLK
	pub fn set_block_size(&mut self, block_size: Option<u32>) {
		self.block_size = block_size;
	}

	/// Returns the `default_permissions` mount data value.
	#[must_use]
	pub fn default_permissions(&self) -> bool {
		self.default_permissions
	}

	/// Sets the `default_permissions` mount data value.
	///
	/// If true then the kernel will perform its own permission checking
	/// in addition to any permission checks by the filesystem.
	pub fn set_default_permissions(&mut self, default_permissions: bool) {
		self.default_permissions = default_permissions;
	}

	/// Returns the `fd=` mount data value.
	#[must_use]
	pub fn fuse_device_fd(&self) -> Option<u32> {
		self.fuse_device_fd
	}

	/// Sets the `fd=` mount data value.
	///
	/// This option is typically not set by a filesystem, but is instead used
	/// by a library wrapping `mount()`.
	pub fn set_fuse_device_fd(&mut self, fuse_device_fd: Option<u32>) {
		self.fuse_device_fd = fuse_device_fd;
	}

	/// Returns the `group_id=` mount data value.
	#[must_use]
	pub fn group_id(&self) -> Option<u32> {
		self.group_id
	}

	/// Sets the `group_id=` mount data value.
	///
	/// This is the group ID of the mount owner, used by the kernel's permission
	/// checking for `unmount()` and the [`allow_other`] option.
	///
	/// [`allow_other`]: MountOptions::allow_other
	pub fn set_group_id(&mut self, group_id: Option<u32>) {
		self.group_id = group_id;
	}

	/// Returns the `max_read=` mount data value.
	#[must_use]
	pub fn max_read(&self) -> Option<u32> {
		self.max_read
	}

	/// Sets the `max_read=` mount data value.
	///
	/// This is a hint to the kernel about the filesystem's largest supported
	/// read size. The kernel is allowed to send `FUSE_READ` requests with
	/// sizes that exceed this value.
	pub fn set_max_read(&mut self, max_read: Option<u32>) {
		self.max_read = max_read;
	}

	/// Returns the `source` mount parameter.
	#[must_use]
	pub fn mount_source(&self) -> &'a MountSource {
		self.mount_source
	}

	/// Sets the `source` mount parameter.
	///
	/// This is the value passed to `mount()` as the `source` parameter when
	/// mounting a FUSE filesystem.
	///
	/// For `fuseblk` filesystems the mount source must be a block device path.
	pub fn set_mount_source(&mut self, mount_source: &'a MountSource) {
		self.mount_source = mount_source;
	}

	/// Returns the `type` mount parameter.
	#[must_use]
	pub fn mount_type(&self) -> &'a MountType {
		self.mount_type
	}

	/// Sets the `type` mount parameter.
	///
	/// This is the value passed to `mount()` as the `type` parameter when
	/// mounting a FUSE filesystem.
	///
	/// There are two valid mount types: [`FUSE`] for normal FUSE filesystems,
	/// or [`FUSEBLK`] for FUSE-wrapped block devices.
	///
	/// [`FUSE`]: MountType::FUSE
	/// [`FUSEBLK`]: MountType::FUSEBLK
	pub fn set_mount_type(&mut self, mount_type: &'a MountType) {
		self.mount_type = mount_type;
	}

	/// Returns the `rootmode` mount data value.
	#[must_use]
	pub fn root_mode(&self) -> Option<node::Mode> {
		self.root_mode
	}

	/// Sets the `rootmode` mount data value.
	///
	/// This option is the filesystem root's Unix file mode. It will typically
	/// be [`S_IFDIR`] plus appropriate permission bits.
	///
	/// [`S_IFDIR`]: node::Mode::S_IFDIR
	pub fn set_root_mode(&mut self, root_mode: Option<node::Mode>) {
		self.root_mode = root_mode;
	}

	/// Returns the `subtype=` mount data value.
	#[must_use]
	pub fn subtype(&self) -> Option<&'a FuseSubtype> {
		self.subtype
	}

	/// Sets the `subtype=` mount data value.
	///
	/// Kernel APIs such as `/proc/mounts` will report the filesystem type as
	/// `fuse.{subtype}`.
	pub fn set_subtype(&mut self, subtype: Option<&'a FuseSubtype>) {
		self.subtype = subtype;
	}

	/// Returns the `user_id` mount data value.
	#[must_use]
	pub fn user_id(&self) -> Option<u32> {
		self.user_id
	}

	/// Sets the `user_id` mount data value.
	///
	/// This is the user ID of the mount owner, used by the kernel's permission
	/// checking for `unmount()` and the [`allow_other`] option.
	///
	/// [`allow_other`]: MountOptions::allow_other
	pub fn set_user_id(&mut self, user_id: Option<u32>) {
		self.user_id = user_id;
	}
}

impl fmt::Debug for MountOptions<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("MountOptions")
			.field("allow_other", &self.allow_other())
			.field("block_size", &format_args!("{:?}", self.block_size()))
			.field("default_permissions", &self.default_permissions())
			.field(
				"fuse_device_fd",
				&format_args!("{:?}", self.fuse_device_fd()),
			)
			.field("group_id", &format_args!("{:?}", self.group_id()))
			.field("max_read", &format_args!("{:?}", self.max_read()))
			.field("mount_source", &self.mount_source())
			.field("mount_type", &self.mount_type())
			.field("root_mode", &format_args!("{:?}", self.root_mode()))
			.field("subtype", &format_args!("{:?}", self.subtype()))
			.field("user_id", &format_args!("{:?}", self.user_id()))
			.finish()
	}
}

// }}}

// mount_data {{{

/// Helper for formatting the `mount` syscall's `data` parameter.
///
/// For FUSE mounts the mount data is a NUL-terminated string containing
/// comma-separated `key=value` pairs.
///
/// This function formats appropriate mount data for the given [`MountOptions`]
/// into a provided buffer, and returns a slice that can be passed to `mount()`.
///
/// Returns `None` if the mount data would exceed the size of the buffer.
#[must_use]
pub fn mount_data<'a>(
	options: &MountOptions,
	storage: &'a mut [u8],
) -> Option<&'a [u8]> {
	let mut w = BufWriter {
		buf: storage,
		count: 0,
	};
	if write_mount_data(&mut w, options).is_err() {
		return None;
	}
	let count = w.count;
	Some(&storage[..count])
}

fn write_mount_data(w: &mut BufWriter, opts: &MountOptions) -> fmt::Result {
	let comma = ",";
	let mut sep = "";

	// Output fd= first so it's easy to locate in debug logs and strace output.
	if let Some(fuse_device_fd) = opts.fuse_device_fd {
		write!(w, "fd={}", fuse_device_fd)?;
		sep = comma;
	}

	// Other options are written in order by key.
	if opts.allow_other {
		write!(w, "{}allow_other", sep)?;
		sep = comma;
	}
	if let Some(block_size) = opts.block_size {
		write!(w, "{}blksize={}", sep, block_size)?;
		sep = comma;
	}
	if opts.default_permissions {
		write!(w, "{}default_permissions", sep)?;
		sep = comma;
	}
	if let Some(group_id) = opts.group_id {
		write!(w, "{}group_id={}", sep, group_id)?;
		sep = comma;
	}
	if let Some(max_read) = opts.max_read {
		write!(w, "{}max_read={}", sep, max_read)?;
		sep = comma;
	}
	if let Some(root_mode) = opts.root_mode {
		write!(w, "{}rootmode={:o}", sep, root_mode.get())?;
		sep = comma;
	}
	if let Some(subtype) = opts.subtype {
		write!(w, "{}subtype=", sep)?;
		w.write_bytes(subtype.as_cstr().to_bytes())?;
		sep = comma;
	}
	if let Some(user_id) = opts.user_id {
		write!(w, "{}user_id={}", sep, user_id)?;
	}

	// Ensure the output is terminated by NUL. Although the `mount()` data
	// parameter is `void*`, FUSE expects its mount data to be a C string.
	w.write_bytes(&[0])?;
	Ok(())
}

// }}}

// BufWriter {{{

struct BufWriter<'a> {
	buf: &'a mut [u8],
	count: usize,
}

impl BufWriter<'_> {
	fn write_bytes(&mut self, b: &[u8]) -> fmt::Result {
		let avail = &mut self.buf[self.count..];
		if b.len() > avail.len() {
			return Err(fmt::Error);
		}
		avail[..b.len()].copy_from_slice(b);
		self.count += b.len();
		Ok(())
	}
}

impl fmt::Write for BufWriter<'_> {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_bytes(s.as_bytes())
	}
}

// }}}

// https://github.com/rust-lang/rust/issues/102444
fn cstr_is_empty(s: &ffi::CStr) -> bool {
	unsafe { s.as_ptr().read() == 0 }
}
