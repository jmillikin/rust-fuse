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

use core::mem::size_of;

use fuse_testutil::MessageBuilder;

use crate::internal::fuse_kernel;
use crate::server;
use crate::server::decode::RequestDecoder;

fn aligned_slice(buf: &fuse::io::MinReadBuffer) -> crate::io::AlignedSlice {
	let slice = buf.as_aligned_slice().get();
	unsafe { crate::io::AlignedSlice::new_unchecked(slice) }
}

#[test]
fn request_decoder_new() {
	let buf = MessageBuilder::new()
		.set_header(|_| {})
		.push_bytes(&[1, 2, 3, 4, 5, 6, 7, 8, 9])
		.build_aligned();

	let request = server::Request::new(aligned_slice(&buf)).unwrap();
	let decoder = unsafe { RequestDecoder::new_unchecked(request.as_slice()) };

	assert_eq!(
		decoder.consumed,
		size_of::<fuse_kernel::fuse_in_header>() as u32
	);
}

#[test]
fn request_decoder_eof_handling() {
	let buf = MessageBuilder::new()
		.set_header(|_| {})
		.push_bytes(&[10, 20, 30, 40, 50, 60, 70, 80, 90])
		.build_aligned();

	let request = server::Request::new(aligned_slice(&buf)).unwrap();
	let mut decoder = unsafe {
		RequestDecoder::new_unchecked(request.as_slice())
	};

	// OK to read right up to the frame size.
	decoder.next_bytes(8).unwrap();
	assert_eq!(decoder.next_bytes(1), Ok(&[90u8] as &[u8]),);

	// reading past the frame size is an error.
	assert_eq!(
		decoder.next_bytes(1),
		Err(server::RequestError::UnexpectedEof)
	);
}

/*
#[test]
#[cfg_attr(target_pointer_width = "16", ignore)]
fn frame_reader_u32_overflow() {
	assert!(size_of::<usize>() >= size_of::<u32>());

	let mut buf = MessageBuilder::new()
		.set_header(|_| {})
		.push_bytes(&[1, 2, 3, 4, 5, 6, 7, 8, 9])
		.build();

	let giant_buf: &[u8];
	unsafe {
		let len_p = buf.as_mut_ptr() as *mut u32;
		*len_p = 0xFFFFFFFF;
		giant_buf = slice::from_raw_parts(buf.as_ptr(), u32::MAX as usize);
	};

	let mut reader = FrameReader::new(&giant_buf).unwrap();
	reader.consumed = u32::MAX - 1;

	// OK to read right up to the frame size.
	assert_eq!(reader.consume(1), Ok(u32::MAX));

	// catch u32 overflow
	assert_eq!(reader.consume(2), Err(Error::unexpected_eof()));
}
*/

#[test]
fn request_decoder_sized() {
	let buf = MessageBuilder::new()
		.set_header(|_| {})
		.push_bytes(&[1, 2, 3, 4, 5, 6, 7, 8, 9])
		.build_aligned();

	let request = server::Request::new(aligned_slice(&buf)).unwrap();
	let mut decoder = unsafe {
		RequestDecoder::new_unchecked(request.as_slice())
	};

	// [0 .. 4]
	let did_read: &[u8; 4] = decoder.next_sized().unwrap();
	assert_eq!(did_read, &[1, 2, 3, 4]);

	// [4 .. 8]
	let did_read: &[u8; 4] = decoder.next_sized().unwrap();
	assert_eq!(did_read, &[5, 6, 7, 8]);

	// [8 .. 12] hits EOF
	assert_eq!(
		decoder.next_sized::<u32>(),
		Err(server::RequestError::UnexpectedEof)
	);
}

#[test]
fn frame_decoder_bytes() {
	let buf = MessageBuilder::new()
		.set_header(|_| {})
		.push_bytes(&[1, 2, 3, 4, 5, 6, 7, 8, 9])
		.build_aligned();

	let request = server::Request::new(aligned_slice(&buf)).unwrap();
	let mut decoder = unsafe {
		RequestDecoder::new_unchecked(request.as_slice())
	};

	// [0 .. 4)
	let did_read = decoder.next_bytes(4).unwrap();
	assert_eq!(did_read, &[1, 2, 3, 4]);

	// [4 .. 8)
	let did_read = decoder.next_bytes(4).unwrap();
	assert_eq!(did_read, &[5, 6, 7, 8]);

	// [8 .. 12) hits EOF
	assert_eq!(
		decoder.next_bytes(4),
		Err(server::RequestError::UnexpectedEof)
	);
}
