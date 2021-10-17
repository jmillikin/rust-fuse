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

use crate::io::RequestError;
use crate::protocol::common::UnknownRequest;
use crate::server::RequestHeader;

#[allow(unused_variables)]
pub trait ServerHooks {
	fn request(&self, header: &RequestHeader) {}

	fn unknown_request(&self, request: &UnknownRequest) {}

	fn unhandled_request(&self, header: &RequestHeader) {}

	fn request_error(&self, header: &RequestHeader, err: RequestError) {}
}

pub enum NoopServerHooks {}

impl ServerHooks for NoopServerHooks {}
