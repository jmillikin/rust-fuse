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

use core::cmp;
use core::convert::TryFrom;
use core::time;

pub(crate) const MAX_NANOS: u32 = 999_999_999;

pub(crate) fn new_duration(raw_seconds: u64, nanos: u32) -> time::Duration {
	// https://github.com/rust-lang/libs-team/issues/117
	time::Duration::new(
		cmp::min(raw_seconds, i64::MAX as u64),
		cmp::min(nanos, MAX_NANOS),
	)
}

pub(crate) fn split_duration(d: time::Duration) -> (u64, u32) {
	let (seconds, nanos) = match i64::try_from(d.as_secs()) {
		Ok(seconds) => (seconds, d.subsec_nanos()),
		Err(_) => (i64::MAX, 0),
	};
	(seconds as u64, nanos)
}
