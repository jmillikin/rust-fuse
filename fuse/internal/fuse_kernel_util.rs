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

macro_rules! fuse_kernel_decls {
	(
		$(
			$(#[$meta:meta])*
			$( # )? $decl_keyword:ident $name:ident $decl:tt $( ; )?
		)+
	) => {
		$(
			fuse_kernel_decl! { $decl_keyword $name $decl }
		)+
	};
}

macro_rules! fuse_kernel_decl {
	(define $name:ident $decl:tt) => {
		//fuse_kernel_constants! { #define $name $decl }
		#[allow(unused_parens)]
		pub(crate) const $name: fuse_kernel_const_ty!($name) = fuse_kernel_const_val!($decl);
	};
	(enum $name:ident $decl:tt) => {
		fuse_kernel_enum! { $name $decl }
	};
	(struct $name:ident $decl:tt) => {
		fuse_kernel_struct! { $name $decl }
	};
}

macro_rules! fuse_kernel_const_ty {
	(FUSE_COMPAT_22_INIT_OUT_SIZE) => {usize};
	(FUSE_COMPAT_ATTR_OUT_SIZE) => { usize };
	(FUSE_COMPAT_ENTRY_OUT_SIZE) => { usize };
	(FUSE_COMPAT_INIT_OUT_SIZE) => {usize};
	(FUSE_COMPAT_MKNOD_IN_SIZE) => { usize };
	(FUSE_COMPAT_SETXATTR_IN_SIZE) => {usize};
	(FUSE_COMPAT_STATFS_SIZE) => {usize};
	(FUSE_COMPAT_WRITE_IN_SIZE) => {usize};
	(FUSE_HAS_INODE_DAX) => { u64 };
	(FUSE_IOCTL_MAX_IOV) => { usize };
	(FUSE_MIN_READ_BUFFER) => { usize };
	(FUSE_ROOT_ID) => { u64 };
	(FUSE_SECURITY_CTX) => { u64 };
	(FUSE_SETUPMAPPING_FLAG_READ) => { u64 };
	(FUSE_SETUPMAPPING_FLAG_WRITE) => { u64 };
	($name:ident) => { u32 };
}

macro_rules! fuse_kernel_const_val {
	( ( 1ULL << $offset:literal ) ) => { 1u64 << $offset };
	( ( 1ull << $offset:literal ) ) => { 1u64 << $offset };
	($val:expr) => { $val }
}

macro_rules! fuse_kernel_enum {
	($name:ident { $( $item_name:ident = $item_val:expr , )+ } ) => {
		#[repr(transparent)]
		#[derive(Copy, Clone, PartialEq, Eq)]
		pub(crate) struct $name(pub(crate) u32);

		$(
			pub(crate) const $item_name: $name = $name($item_val);
		)*

		impl core::fmt::Debug for $name {
			fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
				match self {
				$(
					$name($item_val) => fmt.write_str(stringify!($item_name)),
				)*
					_ => write!(fmt, concat!(stringify!($name), "({})"), self.0),
				}
			}
		}
	};
}

macro_rules! fuse_kernel_struct {
	($name:ident { $( $i1:ident $i2:ident $( $i3:ident )? $( [ $( $arr_size:literal )? ] )? ; )+ }) => {
		fuse_kernel_struct_inner!($name {
			$(
				$( $i3 )? $i2 $i1 $( [ $( $arr_size )? ] )? ;
			)+
		});
	};
}

macro_rules! fuse_kernel_struct_inner {
	($name:ident { $( $field_name:ident $field_c_type:ident $( $i3:ident )? $( [ $( $arr_size:literal )? ] )? ; )+ }) => {
		#[repr(C)]
		#[derive(Clone, Copy, Debug)]
		pub(crate) struct $name {
			$(
				pub(crate) $field_name: fuse_kernel_type!($field_c_type $( [ $( $arr_size )? ] )? ),
			)+
		}
		impl $name {
			pub(crate) fn zeroed() -> Self {
				unsafe { core::mem::zeroed() }
			}
		}
	};
}

macro_rules! fuse_kernel_type {
	(char) => { u8 };
	(uint16_t) => { u16 };
	(int32_t)  => { i32 };
	(uint32_t) => { u32 };
	(int64_t)  => { i64 };
	(uint64_t) => { u64 };
	($t:ident) => { $t };
	($t:ident [ ]) => { [ fuse_kernel_type!($t) ; 0 ] };
	($t:ident [ $arr_size:literal ]) => { [ fuse_kernel_type!($t) ; $arr_size ] };
}
