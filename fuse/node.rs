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

//! Filesystem nodes and node attributes.

use core::fmt;
use core::num;
use core::ops;
use core::time;

use crate::internal::debug;
use crate::internal::fuse_kernel;
use crate::internal::timestamp;


// Id {{{

/// Node IDs are per-mount unique identifiers for filesystem nodes.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
	bits: num::NonZeroU64,
}

const FUSE_ROOT_ID: Id = unsafe {
	Id::new_unchecked(fuse_kernel::FUSE_ROOT_ID)
};

impl Id {
	/// The node ID of the root directory.
	pub const ROOT: Id = FUSE_ROOT_ID;

	/// Creates a new `Id` if the given value is not zero.
	#[inline]
	#[must_use]
	pub const fn new(id: u64) -> Option<Id> {
		match num::NonZeroU64::new(id) {
			Some(id) => Some(Id { bits: id }),
			None => None,
		}
	}

	/// Creates a new `Id` without checking that the given value is non-zero.
	///
	/// # Safety
	///
	/// The value must not be zero.
	///
	/// The `Id` struct is a wrapper around [`NonZeroU64`](num::NonZeroU64),
	/// so passing zero to this function is undefined behavior.
	#[inline]
	#[must_use]
	pub const unsafe fn new_unchecked(id: u64) -> Id {
		Self {
			bits: num::NonZeroU64::new_unchecked(id),
		}
	}

	/// Returns the node ID as a primitive integer.
	#[inline]
	#[must_use]
	pub const fn get(&self) -> u64 {
		self.bits.get()
	}

	/// Returns whether the node ID is [`Id::ROOT`].
	#[inline]
	#[must_use]
	pub const fn is_root(&self) -> bool {
		self.bits.get() == fuse_kernel::FUSE_ROOT_ID
	}
}

impl fmt::Debug for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::Binary for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for Id {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

// }}}

// Name {{{

/// Errors that may occur when validating a node name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NameError {
	/// The input is empty.
	Empty,
	/// The input contains `NUL`.
	ContainsNul,
	/// The input contains `'/'`.
	ContainsSlash,
	/// The input length in bytes exceeds [`Name::MAX_LEN`].
	ExceedsMaxLen,
}

/// A borrowed filesystem node name.
///
/// This type represents a borrowed reference to an array of bytes containing
/// the name of a filesystem node. It can be constructed safely from a `&str`
/// or `&[u8]` slice.
///
/// An instance of this type is a static guarantee that the underlying byte
/// array is non-empty, is less than [`Name::MAX_LEN`] bytes in length, and
/// does not contain a forbidden character (`NUL` or `'/'`).
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Name {
	bytes: [u8]
}

#[cfg(target_os = "freebsd")]
const NAME_MAX: usize = 255;

#[cfg(target_os = "freebsd")]
macro_rules! node_name_max_len {
	() => { Some(NAME_MAX) }
}

#[cfg(target_os = "linux")]
const FUSE_NAME_MAX: usize = 1024;

#[cfg(target_os = "linux")]
macro_rules! node_name_max_len {
	() => { Some(FUSE_NAME_MAX) }
}

impl Name {
	/// The maximum length of a node name, in bytes.
	///
	/// This value is platform-specific. If `None`, then the platform does not
	/// impose a maximum length on node names.
	///
	/// | Platform | Symbolic constant | Value |
	/// |----------|-------------------|-------|
	/// | FreeBSD  | `NAME_MAX`        | 255   |
	/// | Linux    | `FUSE_NAME_MAX`   | 1024  |
	///
	pub const MAX_LEN: Option<usize> = node_name_max_len!();

	/// Attempts to reborrow a string as a node name.
	///
	/// # Errors
	///
	/// Returns an error if the string is empty, is longer than
	/// [`Name::MAX_LEN`] bytes, or contains a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	pub fn new(name: &str) -> Result<&Name, NameError> {
		Self::from_bytes(name.as_bytes())
	}

	/// Reborrows a string as a node name, without validation.
	///
	/// # Safety
	///
	/// The provided string must be non-empty, must be no longer than
	/// [`Name::MAX_LEN`] bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub unsafe fn new_unchecked(name: &str) -> &Name {
		Self::from_bytes_unchecked(name.as_bytes())
	}

	/// Attempts to reborrow a byte slice as a node name.
	///
	/// # Errors
	///
	/// Returns an error if the slice is empty, is longer than
	/// [`Name::MAX_LEN`] bytes, or contains a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	pub fn from_bytes(bytes: &[u8]) -> Result<&Name, NameError> {
		if bytes.is_empty() {
			return Err(NameError::Empty);
		}
		if let Some(max_len) = Name::MAX_LEN {
			if bytes.len() > max_len {
				return Err(NameError::ExceedsMaxLen);
			}
		}
		for &byte in bytes {
			if byte == 0 {
				return Err(NameError::ContainsNul);
			}
			if byte == b'/' {
				return Err(NameError::ContainsSlash);
			}
		}
		Ok(unsafe { Self::from_bytes_unchecked(bytes) })
	}

	/// Reborrows a byte slice as a node name, without validation.
	///
	/// # Safety
	///
	/// The provided slice must be non-empty, must be no longer than
	/// [`Name::MAX_LEN`] bytes, and must not contain a forbidden character
	/// (`NUL` or `'/'`).
	#[inline]
	#[must_use]
	pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Name {
		&*(bytes as *const [u8] as *const Name)
	}

	/// Converts this `Name` to a byte slice.
	#[inline]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Attempts to convert this `Name` to a `&str`.
	///
	/// # Errors
	///
	/// Returns an error if the name is not UTF-8.
	#[inline]
	pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.bytes)
	}
}

impl fmt::Debug for Name {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		debug::bytes(&self.bytes).fmt(fmt)
	}
}

impl PartialEq<str> for Name {
	fn eq(&self, other: &str) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<[u8]> for Name {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes().eq(other)
	}
}

impl PartialEq<Name> for str {
	fn eq(&self, other: &Name) -> bool {
		self.as_bytes().eq(other.as_bytes())
	}
}

impl PartialEq<Name> for [u8] {
	fn eq(&self, other: &Name) -> bool {
		self.eq(other.as_bytes())
	}
}

// }}}

// Mode {{{

/// Representation of Unix file modes.
///
/// Unix file modes are a bitmask comprising a [`Type`], permission bits,
/// and additional flags such as the [sticky bit].
///
/// The lowest 9 mode bits (the "permission bits") are standardized by POSIX.
/// Presence and interpretation of other bits is platform-specific.
///
/// [sticky bit]: https://en.wikipedia.org/wiki/Sticky_bit
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mode {
	bits: u32,
}

impl Mode {
	/// Mode mask for a [`Directory`](Type::Directory) node.
	pub const S_IFDIR: Mode = Type::Directory.as_mode();
	/// Mode mask for a [`CharacterDevice`](Type::CharacterDevice) node.
	pub const S_IFCHR: Mode = Type::CharacterDevice.as_mode();
	/// Mode mask for a [`BlockDevice`](Type::BlockDevice) node.
	pub const S_IFBLK: Mode = Type::BlockDevice.as_mode();
	/// Mode mask for a [`Regular`](Type::Regular) node.
	pub const S_IFREG: Mode = Type::Regular.as_mode();
	/// Mode mask for a [`Symlink`](Type::Symlink) node.
	pub const S_IFLNK: Mode = Type::Symlink.as_mode();
	/// Mode mask for a [`Socket`](Type::Socket) node.
	pub const S_IFSOCK: Mode = Type::Socket.as_mode();
	/// Mode mask for a [`NamedPipe`](Type::NamedPipe) node.
	pub const S_IFIFO: Mode = Type::NamedPipe.as_mode();
}

impl Mode {
	/// Creates a new `Mode` with the given value.
	#[inline]
	#[must_use]
	pub const fn new(mode: u32) -> Mode {
		Self { bits: mode }
	}

	/// Returns the mode as a primitive integer.
	#[inline]
	#[must_use]
	pub const fn get(self) -> u32 {
		self.bits
	}

	/// Returns the permission bits set in the mode.
	///
	/// The permission bits are the lowest 9 bits (mask `0o777`).
	#[inline]
	#[must_use]
	pub const fn permissions(self) -> u32 {
		self.bits & 0o777
	}

	#[inline]
	#[must_use]
	pub(crate) const fn type_bits(self) -> u32 {
		self.bits & S_IFMT
	}
}

impl fmt::Debug for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{:#o}", self.bits)
	}
}

impl fmt::Binary for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::LowerHex for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl fmt::UpperHex for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.bits.fmt(fmt)
	}
}

impl ops::BitOr<u32> for Mode {
	type Output = Mode;

	fn bitor(self, rhs: u32) -> Mode {
		Mode {
			bits: self.bits | rhs,
		}
	}
}

// }}}

// Type {{{

/// Representation of Unix file types.
///
/// A Unix file type identifies the purpose and capabilities of a filesystem
/// node. The POSIX standard describes seven file types; platforms may add
/// additional types as extensions.
///
/// Most FUSE filesystems will only create nodes of type `Directory`, `Regular`,
/// or `Symlink`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Type {
	/// A [block device].
	///
	/// [block device]: https://en.wikipedia.org/wiki/Device_file#Block_devices
	BlockDevice,

	/// A [character device].
	///
	/// [character device]: https://en.wikipedia.org/wiki/Device_file#Character_devices
	CharacterDevice,

	/// A directory.
	Directory,

	/// A [named pipe].
	///
	/// [named pipe]: https://en.wikipedia.org/wiki/Named_pipe
	NamedPipe,

	/// A regular file.
	Regular,

	/// A [Unix domain socket].
	///
	/// [Unix domain socket]: https://en.wikipedia.org/wiki/Unix_domain_socket
	Socket,

	/// A [symbolic link].
	///
	/// [symbolic link]: https://en.wikipedia.org/wiki/Symbolic_link
	Symlink,
}

const S_IFMT: u32 = 0xF000;

impl Type {
	const DT_FIFO: u32 =  1;
	const DT_CHR:  u32 =  2;
	const DT_DIR:  u32 =  4;
	const DT_BLK:  u32 =  6;
	const DT_REG:  u32 =  8;
	const DT_LNK:  u32 = 10;
	const DT_SOCK: u32 = 12;

	/// Returns the `Type` contained in a Unix file mode.
	///
	/// Returns `None` if the mode's type bits are an unknown value. To support
	/// platform-specific file types, the user should inspect the mode directly.
	#[inline]
	#[must_use]
	pub const fn from_mode(mode: Mode) -> Option<Type> {
		Type::from_bits(mode.type_bits() >> 12)
	}

	#[inline]
	#[must_use]
	pub const fn as_mode(self) -> Mode {
		Mode::new(self.as_bits() << 12)
	}

	#[inline]
	#[must_use]
	pub(crate) const fn as_bits(self) -> u32 {
		match self {
			Self::NamedPipe       => Self::DT_FIFO,
			Self::CharacterDevice => Self::DT_CHR,
			Self::Directory       => Self::DT_DIR,
			Self::BlockDevice     => Self::DT_BLK,
			Self::Regular         => Self::DT_REG,
			Self::Symlink         => Self::DT_LNK,
			Self::Socket          => Self::DT_SOCK,
		}
	}

	#[inline]
	#[must_use]
	pub(crate) const fn from_bits(bits: u32) -> Option<Type> {
		match bits {
			Self::DT_FIFO => Some(Self::NamedPipe),
			Self::DT_CHR  => Some(Self::CharacterDevice),
			Self::DT_DIR  => Some(Self::Directory),
			Self::DT_BLK  => Some(Self::BlockDevice),
			Self::DT_REG  => Some(Self::Regular),
			Self::DT_LNK  => Some(Self::Symlink),
			Self::DT_SOCK => Some(Self::Socket),
			_ => None,
		}
	}
}

impl From<Type> for Mode {
	fn from(node_type: Type) -> Mode {
		node_type.as_mode()
	}
}

// }}}

// Attributes {{{

/// Attributes of a filesystem node.
#[derive(Clone, Copy)]
pub struct Attributes {
	raw: fuse_kernel::fuse_attr,
}

impl Attributes {
	/// Creates a new `Attributes` for a node with the given ID.
	#[inline]
	#[must_use]
	pub fn new(node_id: Id) -> Attributes {
		Self {
			raw: fuse_kernel::fuse_attr {
				ino: node_id.get(),
				..fuse_kernel::fuse_attr::zeroed()
			},
		}
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref(raw: &fuse_kernel::fuse_attr) -> &Self {
		let raw_ptr = raw as *const fuse_kernel::fuse_attr;
		&*(raw_ptr.cast::<Attributes>())
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref_mut(
		raw: &mut fuse_kernel::fuse_attr,
	) -> &mut Self {
		let raw_ptr = raw as *mut fuse_kernel::fuse_attr;
		&mut *(raw_ptr.cast::<Attributes>())
	}

	/// Returns the per-mount unique identifier of the node.
	#[inline]
	#[must_use]
	pub fn node_id(&self) -> Id {
		unsafe { Id::new_unchecked(self.raw.ino) }
	}

	/// Returns the node's mode, including type and permissions.
	#[inline]
	#[must_use]
	pub fn mode(&self) -> Mode {
		Mode::new(self.raw.mode)
	}

	/// Sets the node's mode, including type and permissions.
	#[inline]
	pub fn set_mode(&mut self, mode: Mode) {
		self.raw.mode = mode.get();
	}

	/// Returns the node's size.
	#[inline]
	#[must_use]
	pub fn size(&self) -> u64 {
		self.raw.size
	}

	/// Sets the node's size.
	#[inline]
	pub fn set_size(&mut self, size: u64) {
		self.raw.size = size;
	}

	/// Returns the node's last access time.
	#[inline]
	#[must_use]
	pub fn atime(&self) -> crate::UnixTime {
		unsafe {
			crate::UnixTime::from_timespec_unchecked(
				self.raw.atime,
				self.raw.atimensec,
			)
		}
	}

	/// Sets the node's last access time.
	#[inline]
	pub fn set_atime(&mut self, atime: crate::UnixTime) {
		(self.raw.atime, self.raw.atimensec) = atime.as_timespec();
	}

	/// Returns the node's last content modification time.
	#[inline]
	#[must_use]
	pub fn mtime(&self) -> crate::UnixTime {
		unsafe {
			crate::UnixTime::from_timespec_unchecked(
				self.raw.mtime,
				self.raw.mtimensec,
			)
		}
	}

	/// Sets the node's last content modification time.
	#[inline]
	pub fn set_mtime(&mut self, mtime: crate::UnixTime) {
		(self.raw.mtime, self.raw.mtimensec) = mtime.as_timespec();
	}

	/// Returns the node's last status change time.
	#[inline]
	#[must_use]
	pub fn ctime(&self) -> crate::UnixTime {
		unsafe {
			crate::UnixTime::from_timespec_unchecked(
				self.raw.ctime,
				self.raw.ctimensec,
			)
		}
	}

	/// Sets the node's last status change time.
	#[inline]
	pub fn set_ctime(&mut self, ctime: crate::UnixTime) {
		(self.raw.ctime, self.raw.ctimensec) = ctime.as_timespec();
	}

	/// Returns the node's link count.
	#[inline]
	#[must_use]
	pub fn link_count(&self) -> u32 {
		self.raw.nlink
	}

	/// Sets the node's link count.
	///
	/// In general nodes accessible via `FUSE_LOOKUP` should have a non-zero
	/// link count. A link count of zero means the node has been removed but is
	/// still referenced by an open file handle.
	#[inline]
	pub fn set_link_count(&mut self, link_count: u32) {
		self.raw.nlink = link_count;
	}

	/// Returns the node's owning user ID.
	#[inline]
	#[must_use]
	pub fn user_id(&self) -> u32 {
		self.raw.uid
	}

	/// Sets the node's owning user ID.
	#[inline]
	pub fn set_user_id(&mut self, user_id: u32) {
		self.raw.uid = user_id;
	}

	/// Returns the node's owning group ID.
	#[inline]
	#[must_use]
	pub fn group_id(&self) -> u32 {
		self.raw.gid
	}

	/// Sets the node's owning group ID.
	#[inline]
	pub fn set_group_id(&mut self, group_id: u32) {
		self.raw.gid = group_id;
	}

	/// Returns the [device number] of a [`BlockDevice`] or [`CharacterDevice`]
	/// node.
	///
	/// [device number]: https://www.kernel.org/doc/html/latest/admin-guide/devices.html
	/// [`BlockDevice`]: Type::BlockDevice
	/// [`CharacterDevice`]: Type::CharacterDevice
	#[inline]
	#[must_use]
	pub fn device_number(&self) -> u32 {
		self.raw.rdev
	}

	/// Sets the [device number] of a [`BlockDevice`] or [`CharacterDevice`]
	/// node.
	///
	/// [device number]: https://www.kernel.org/doc/html/latest/admin-guide/devices.html
	/// [`BlockDevice`]: Type::BlockDevice
	/// [`CharacterDevice`]: Type::CharacterDevice
	#[inline]
	pub fn set_device_number(&mut self, device_number: u32) {
		self.raw.rdev = device_number;
	}

	/// Returns the number of blocks allocated by the node.
	#[inline]
	#[must_use]
	pub fn block_count(&self) -> u64 {
		self.raw.blocks
	}

	/// Sets the number of blocks allocated by the node.
	#[inline]
	pub fn set_block_count(&mut self, block_count: u64) {
		self.raw.blocks = block_count;
	}

	/// Returns the block size of the node.
	#[inline]
	#[must_use]
	pub fn block_size(&self) -> u32 {
		self.raw.blksize
	}

	/// Sets the block size of the node.
	#[inline]
	pub fn set_block_size(&mut self, block_size: u32) {
		self.raw.blksize = block_size;
	}

	/// Returns whether the node is the root of a submount.
	#[inline]
	#[must_use]
	pub fn flag_submount(&self) -> bool {
		self.flags().get(AttributeFlag::FUSE_ATTR_SUBMOUNT)
	}

	/// Sets whether the node is the root of a submount.
	#[inline]
	pub fn set_flag_submount(&mut self, is_submount: bool) {
		self.flags_mut().set_to(AttributeFlag::FUSE_ATTR_SUBMOUNT, is_submount)
	}

	/// Returns whether [DAX] is enabled for the node.
	///
	/// [DAX]: https://www.kernel.org/doc/html/latest/filesystems/dax.html
	#[inline]
	#[must_use]
	pub fn flag_dax(&self) -> bool {
		self.flags().get(AttributeFlag::FUSE_ATTR_DAX)
	}

	/// Sets whether [DAX] is enabled for the node.
	///
	/// [DAX]: https://www.kernel.org/doc/html/latest/filesystems/dax.html
	#[inline]
	pub fn set_flag_dax(&mut self, enable_dax: bool) {
		self.flags_mut().set_to(AttributeFlag::FUSE_ATTR_DAX, enable_dax)
	}

	#[inline]
	#[must_use]
	fn flags(&self) -> AttributeFlags {
		AttributeFlags {
			bits: self.raw.flags,
		}
	}

	#[inline]
	#[must_use]
	fn flags_mut(&mut self) -> &mut AttributeFlags {
		AttributeFlags::reborrow_mut(&mut self.raw.flags)
	}
}

impl fmt::Debug for Attributes {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Attributes")
			.field("node_id", &self.node_id())
			.field("mode", &self.mode())
			.field("size", &self.size())
			.field("atime", &format_args!("{:?}", self.atime()))
			.field("mtime", &format_args!("{:?}", self.mtime()))
			.field("ctime", &format_args!("{:?}", self.ctime()))
			.field("link_count", &self.link_count())
			.field("user_id", &self.user_id())
			.field("group_id", &self.group_id())
			.field("device_number", &self.device_number())
			.field("block_count", &self.block_count())
			.field("block_size", &self.block_size())
			.field("flags", &self.flags())
			.finish()
	}
}

/// Optional flags set on [`Attributes`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct AttributeFlags {
	bits: u32,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct AttributeFlag {
	mask: u32,
}

mod attr_flags {
	use crate::internal::fuse_kernel;
	bitflags!(AttributeFlag, AttributeFlags, u32, {
		FUSE_ATTR_SUBMOUNT = fuse_kernel::FUSE_ATTR_SUBMOUNT;
		FUSE_ATTR_DAX = fuse_kernel::FUSE_ATTR_DAX;
	});
}

// }}}

// FuseAttrOut {{{

#[derive(Clone, Copy)]
pub(crate) struct FuseAttrOut {
	raw: fuse_kernel::fuse_attr_out,
}

impl FuseAttrOut {
	#[inline]
	#[must_use]
	pub(crate) fn new(attributes: Attributes) -> FuseAttrOut {
		Self {
			raw: fuse_kernel::fuse_attr_out {
				attr_valid: 0,
				attr_valid_nsec: 0,
				dummy: 0,
				attr: attributes.raw,
			},
		}
	}

	#[inline]
	#[must_use]
	pub(crate) fn attributes(&self) -> &Attributes {
		unsafe { Attributes::from_ref(&self.raw.attr) }
	}

	#[inline]
	#[must_use]
	pub(crate) fn attributes_mut(&mut self) -> &mut Attributes {
		unsafe { Attributes::from_ref_mut(&mut self.raw.attr) }
	}

	#[inline]
	#[must_use]
	pub(crate) fn cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.raw.attr_valid, self.raw.attr_valid_nsec)
	}

	#[inline]
	pub(crate) fn set_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.raw.attr_valid = seconds;
		self.raw.attr_valid_nsec = nanos;
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_v7p9(&self) -> &fuse_kernel::fuse_attr_out {
		let self_ptr = self as *const FuseAttrOut;
		unsafe { &*(self_ptr.cast::<fuse_kernel::fuse_attr_out>()) }
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_v7p1(
		&self,
	) -> &[u8; fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE] {
		let self_ptr = self as *const FuseAttrOut;
		const OUT_SIZE: usize = fuse_kernel::FUSE_COMPAT_ATTR_OUT_SIZE;
		unsafe { &*(self_ptr.cast::<[u8; OUT_SIZE]>()) }
	}
}

// }}}

// Entry {{{

/// Cacheable directory entry for a filesystem node.
#[derive(Clone, Copy)]
pub struct Entry {
	raw: fuse_kernel::fuse_entry_out,
}

impl Entry {
	/// Creates a new `Entry` for a node with the given attributes.
	#[inline]
	#[must_use]
	pub fn new(attributes: Attributes) -> Entry {
		Self {
			raw: fuse_kernel::fuse_entry_out {
				nodeid: attributes.raw.ino,
				attr: attributes.raw,
				..fuse_kernel::fuse_entry_out::zeroed()
			},
		}
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref(raw: &fuse_kernel::fuse_entry_out) -> &Self {
		let raw_ptr = raw as *const fuse_kernel::fuse_entry_out;
		&*(raw_ptr.cast::<Entry>())
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_ref_mut(
		raw: &mut fuse_kernel::fuse_entry_out,
	) -> &mut Self {
		let raw_ptr = raw as *mut fuse_kernel::fuse_entry_out;
		&mut *(raw_ptr.cast::<Entry>())
	}

	#[inline]
	#[must_use]
	pub(crate) fn into_entry_out(self) -> fuse_kernel::fuse_entry_out {
		self.raw
	}

	/// Returns the generation number for this entry.
	#[inline]
	#[must_use]
	pub fn generation(&self) -> u64 {
		self.raw.generation
	}

	/// Sets the generation number for this entry.
	#[inline]
	pub fn set_generation(&mut self, generation: u64) {
		self.raw.generation = generation;
	}

	/// Returns the node attributes for this entry.
	#[inline]
	#[must_use]
	pub fn attributes(&self) -> &Attributes {
		unsafe { Attributes::from_ref(&self.raw.attr) }
	}

	/// Returns a mutable reference to the node attributes for this entry.
	#[inline]
	#[must_use]
	pub fn attributes_mut(&mut self) -> &mut Attributes {
		unsafe { Attributes::from_ref_mut(&mut self.raw.attr) }
	}

	/// Returns the lookup cache timeout for this entry.
	#[inline]
	#[must_use]
	pub fn cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.raw.entry_valid, self.raw.entry_valid_nsec)
	}

	/// Sets the lookup cache timeout for this entry.
	#[inline]
	pub fn set_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.raw.entry_valid = seconds;
		self.raw.entry_valid_nsec = nanos;
	}

	/// Returns the attribute cache timeout for this entry.
	#[inline]
	#[must_use]
	pub fn attribute_cache_timeout(&self) -> time::Duration {
		timestamp::new_duration(self.raw.attr_valid, self.raw.attr_valid_nsec)
	}

	/// Sets the attribute cache timeout for this entry.
	#[inline]
	pub fn set_attribute_cache_timeout(&mut self, timeout: time::Duration) {
		let (seconds, nanos) = timestamp::split_duration(timeout);
		self.raw.attr_valid = seconds;
		self.raw.attr_valid_nsec = nanos;
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_v7p9(&self) -> &fuse_kernel::fuse_entry_out {
		let self_ptr = self as *const Entry;
		unsafe { &*(self_ptr.cast::<fuse_kernel::fuse_entry_out>()) }
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_v7p1(
		&self,
	) -> &[u8; fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE] {
		let self_ptr = self as *const Entry;
		const OUT_SIZE: usize = fuse_kernel::FUSE_COMPAT_ENTRY_OUT_SIZE;
		unsafe { &*(self_ptr.cast::<[u8; OUT_SIZE]>()) }
	}
}

impl fmt::Debug for Entry {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Entry")
			.field("generation", &self.generation())
			.field("attributes", &self.attributes())
			.field("cache_timeout", &self.cache_timeout())
			.field("attribute_cache_timeout", &self.attribute_cache_timeout())
			.finish()
	}
}

// }}}
