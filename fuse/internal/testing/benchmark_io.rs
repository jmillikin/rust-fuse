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

use fuse::server;
use fuse::server::fuse_rpc;
use fuse::server::io::Socket;

struct FakeSocket {
	buf: Vec<u8>,
}

impl server::io::FuseSocket for FakeSocket {}

impl server::io::Socket for FakeSocket {
	type Error = std::io::Error;

	fn recv(
		&self,
		buf: &mut [u8],
	) -> Result<usize, fuse::server::io::RecvError<Self::Error>> {
		let copy_dst = &mut buf[..self.buf.len()];
		copy_dst.copy_from_slice(&self.buf);
		Ok(self.buf.len())
	}

	fn send(
		&self,
		_buf: fuse::io::SendBuf,
	) -> Result<(), fuse::server::io::SendError<Self::Error>> {
		Ok(())
	}
}

struct FakeHandlers {}

impl fuse_rpc::Handlers<FakeSocket> for FakeHandlers {
	fn read(
		&self,
		call: fuse_rpc::Call<FakeSocket>,
		_request: &fuse::ReadRequest,
	) -> fuse_rpc::FuseResult<fuse::ReadResponse, std::io::Error> {
		let resp = fuse::ReadResponse::from_bytes(&[0u8; 4096]);
		call.respond_ok(&resp)
	}

	fn write(
		&self,
		call: fuse_rpc::Call<FakeSocket>,
		request: &fuse::WriteRequest,
	) -> fuse_rpc::FuseResult<fuse::WriteResponse, std::io::Error> {
		let mut resp = fuse::WriteResponse::new();
		resp.set_size(request.value().len() as u32);
		call.respond_ok(&resp)
	}
}

fn benchmark_read(c: &mut criterion::Criterion) {
	let buf = fuse_testutil::MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_READ;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_read_in {
			fh: 123,
			offset: 45,
			size: 12,
			read_flags: 0,
			lock_owner: 0,
			flags: 67,
			padding: 0,
		})
		.build_aligned();

	let socket = FakeSocket {
		buf: buf.as_slice().to_vec(),
	};
	let handlers = FakeHandlers {};

	let mut init = fuse::FuseInitResponse::new();
	init.set_version(fuse::Version::new(7, u32::MAX));
	let req_opts = fuse::server::FuseRequestOptions::from_init_response(&init);
	let dispatcher = fuse_rpc::Dispatcher::new(&socket, &handlers, req_opts);

	let request_buf = server::Request::new(buf.as_aligned_slice()).unwrap();

	c.bench_function("read_end_to_end", |b| {
		let mut recv_buf = fuse::io::MinReadBuffer::new();
		b.iter(|| {
			let recv_len = socket.recv(recv_buf.as_slice_mut()).unwrap();
			let recv_buf = recv_buf.as_aligned_slice().truncate(recv_len);
			let request = server::Request::new(recv_buf).unwrap();
			dispatcher.dispatch(request)
		})
	});

	c.bench_function("read_decode", |b| {
		b.iter(|| {
			use fuse::server::FuseRequest;
			fuse::ReadRequest::from_request(request_buf, req_opts)
		})
	});

	c.bench_function("read_dispatch", |b| {
		b.iter(|| dispatcher.dispatch(request_buf))
	});
}

fn benchmark_write(c: &mut criterion::Criterion) {
	let buf = fuse_testutil::MessageBuilder::new()
		.set_header(|h| {
			h.opcode = fuse_kernel::FUSE_WRITE;
			h.nodeid = 123;
		})
		.push_sized(&fuse_kernel::fuse_write_in {
			fh: 123,
			offset: 45,
			size: 4096,
			write_flags: 0,
			lock_owner: 0,
			flags: 67,
			padding: 0,
		})
		.push_bytes(&[0u8; 4096])
		.build_aligned();

	let socket = FakeSocket {
		buf: buf.as_slice().to_vec(),
	};
	let handlers = FakeHandlers {};

	let mut init = fuse::FuseInitResponse::new();
	init.set_version(fuse::Version::new(7, u32::MAX));
	let req_opts = fuse::server::FuseRequestOptions::from_init_response(&init);
	let dispatcher = fuse_rpc::Dispatcher::new(&socket, &handlers, req_opts);

	let request_buf = server::Request::new(buf.as_aligned_slice()).unwrap();

	c.bench_function("write_end_to_end", |b| {
		let mut recv_buf = fuse::io::MinReadBuffer::new();
		b.iter(|| {
			let recv_len = socket.recv(recv_buf.as_slice_mut()).unwrap();
			let recv_buf = recv_buf.as_aligned_slice().truncate(recv_len);
			let request = server::Request::new(recv_buf).unwrap();
			dispatcher.dispatch(request)
		})
	});

	c.bench_function("write_decode", |b| {
		b.iter(|| {
			use fuse::server::FuseRequest;
			fuse::WriteRequest::from_request(request_buf, req_opts)
		})
	});

	c.bench_function("write_dispatch", |b| {
		b.iter(|| dispatcher.dispatch(request_buf))
	});
}

fn main() {
	let mut criterion = criterion::Criterion::default().configure_from_args();

	benchmark_read(&mut criterion);
	benchmark_write(&mut criterion);

	criterion.final_summary();
}
