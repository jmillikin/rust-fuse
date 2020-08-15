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

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Opcode(pub u32);

macro_rules! enum_fuse_opcode {
    ($( $(#[$meta:meta])* $name:ident = $val:expr,)+ ) => {
        use core::fmt;
        pub use crate::internal::fuse_kernel::Opcode;
        $(
            $(#[$meta])*
            pub const $name: Opcode = Opcode($val);
        )*
        impl fmt::Debug for Opcode {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                match self {
                $(
                    $(#[$meta])*
                    Opcode($val) => fmt.write_str(&stringify!($name)),
                )*
                    Opcode(x) => write!(fmt, "{}", x),
                }
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
macro_rules! _IOR {
	(229, 0, uint32_t) => {
		2147804416u32
	};
}
