// Copyright 2021 John Millikin and the rust-fuse contributors.
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

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error<IoError> {
	InvalidReply(ReplyError),
	InvalidRequest(RequestError),
	RecvFail(RecvError<IoError>),
	SendFail(SendError<IoError>),
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecvError<IoError> {
	ConnectionClosed,
	Other(IoError),
}

impl<T> From<RecvError<T>> for Error<T> {
	fn from(err: RecvError<T>) -> Error<T> {
		Error::RecvFail(err)
	}
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SendError<IoError> {
	NotFound,
	Other(IoError),
}

impl<T> From<SendError<T>> for Error<T> {
	fn from(err: SendError<T>) -> Error<T> {
		Error::SendFail(err)
	}
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReplyError {
}

impl<E> From<ReplyError> for Error<E> {
	fn from(err: ReplyError) -> Self {
		Error::InvalidReply(err)
	}
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestError {
	InvalidLockType,
	MissingNodeId,
	OpcodeMismatch,
	UnexpectedEof,
}

impl<E> From<RequestError> for Error<E> {
	fn from(err: RequestError) -> Self {
		Error::InvalidRequest(err)
	}
}
