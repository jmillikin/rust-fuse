// Copyright 2021 John Millikin and the rust-fuse contributors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//		 http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::io::{self, Read, Write};

pub fn send_frame(w: &mut impl Write, buf: &[u8]) -> io::Result<()> {
	let len_buf = (buf.len() as u64).to_be_bytes();
	w.write_all(&len_buf)?;
	w.write_all(buf)
}

pub fn send_file(w: &mut impl Write, file: &mut fs::File) -> io::Result<()> {
	let file_len: u64 = file.metadata()?.len();
	let file_len_buf = file_len.to_be_bytes();
	w.write_all(&file_len_buf)?;
	io::copy(file, w)?;
	Ok(())
}

pub fn recv_frame(r: &mut impl Read) -> io::Result<Vec<u8>> {
	let mut len_buf = [0; 8];
	r.read_exact(&mut len_buf)?;
	let len = u64::from_be_bytes(len_buf);
	let mut buf = Vec::new();
	if len > 0 {
		r.take(len).read_to_end(&mut buf)?;
	}
	Ok(buf)
}

pub fn recv_file(r: &mut impl Read, file: &mut fs::File) -> io::Result<()> {
	let mut len_buf = [0; 8];
	r.read_exact(&mut len_buf)?;
	let len = u64::from_be_bytes(len_buf);
	if len > 0 {
		io::copy(&mut r.take(len), file)?;
	}
	Ok(())
}
