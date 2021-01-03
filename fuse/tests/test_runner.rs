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
use std::os::unix::fs::OpenOptionsExt;
use std::process::{Command, Stdio};

fn main() {
	let mut virtio = fs::OpenOptions::new()
		.read(true)
		.write(true)
		.open("/dev/vport0p1")
		.unwrap();
	println!("[rust-fuse] TEST_RUNNER_READY");

	loop {
		let mut client_file = fs::OpenOptions::new()
			.write(true)
			.create_new(true)
			.mode(0o700)
			.open("/rust-fuse/test_client")
			.unwrap();
		frame_io::recv_file(&mut virtio, &mut client_file).unwrap();
		client_file.sync_all().unwrap();
		std::mem::drop(client_file);

		let mut server_file = fs::OpenOptions::new()
			.write(true)
			.create_new(true)
			.mode(0o700)
			.open("/rust-fuse/test_server")
			.unwrap();
		frame_io::recv_file(&mut virtio, &mut server_file).unwrap();
		server_file.sync_all().unwrap();
		std::mem::drop(server_file);

		let mut server = Command::new("/rust-fuse/test_server")
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.expect("Failed to start test_server");

		std::thread::sleep(std::time::Duration::from_secs(1));

		let client_output = Command::new("/rust-fuse/test_client")
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.output()
			.expect("Failed to run test_client");

		let _ = server.kill();
		let server_output = server.wait_with_output().unwrap();

		frame_io::send_frame(&mut virtio, &client_output.stdout).unwrap();
		frame_io::send_frame(&mut virtio, &client_output.stderr).unwrap();
		frame_io::send_frame(&mut virtio, &server_output.stdout).unwrap();
		frame_io::send_frame(&mut virtio, &server_output.stderr).unwrap();

		fs::remove_file("/rust-fuse/test_client").unwrap();
		fs::remove_file("/rust-fuse/test_server").unwrap();
	}
}
