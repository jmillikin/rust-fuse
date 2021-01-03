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
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{mpsc, Once};
use std::thread;
use std::time;

macro_rules! rust_fuse_test_cases {
	($( $testcase:ident )*) => {
		$(
			#[test]
			fn $testcase() {
				run_test(stringify!($testcase))
			}
		)*
	}
}

rust_fuse_test_cases! {
	getxattr
}

struct Qemu {
	process: Child,
	qmp_stream: TcpStream,
	serial_stream: TcpStream,
}

impl Qemu {
	#[allow(dead_code)]
	fn quit(&mut self) -> io::Result<ExitStatus> {
		self.qmp_stream
			.write_all(b"{\"execute\": \"quit\"}\n")
			.unwrap();
		{
			let mut buf = Vec::new();
			let _ = self.qmp_stream.read_to_end(&mut buf);
			println!("qmp buf: {:?}", std::str::from_utf8(&buf));
		}

		self.process.wait()
	}
}

static mut QEMU: Option<Qemu> = None;
static QEMU_INIT: Once = Once::new();

fn get_qemu() -> &'static mut Qemu {
	QEMU_INIT.call_once(|| {
		let qemu = init_qemu().unwrap();
		unsafe {
			QEMU = Some(qemu)
		};
	});
	unsafe { QEMU.as_mut().unwrap() }
}

fn init_qemu() -> io::Result<Qemu> {
	let qmp_sock = TcpListener::bind("127.0.0.1:0")?;
	let qmp_port = qmp_sock.local_addr()?.port();

	let serial_sock = TcpListener::bind("127.0.0.1:0")?;
	let serial_port = serial_sock.local_addr()?.port();

	let mut qemu_process: Child;
	if let Ok(_) = fs::metadata("fuse/tests/x86_64") {
		#[rustfmt::skip]
		let qemu = Command::new("qemu-system-x86_64")
			.arg("-m").arg("512M")
			.arg("-kernel").arg("../linux_kernel/arch/x86_64/boot/bzImage")
			.arg("-initrd").arg("fuse/tests/x86_64/initrd.cpio.gz")
			.arg("-serial").arg("stdio")
			.arg("-chardev").arg(format!("socket,id=foo,host=127.0.0.1,port={}", serial_port))
			.arg("-device").arg("virtio-serial")
			.arg("-device").arg("virtserialport,chardev=foo,id=test0,nr=1")
			.arg("-qmp").arg(format!("tcp:127.0.0.1:{},nodelay", qmp_port))
			.arg("-append").arg("console=ttyS0 rdinit=/bin/init")
			.arg("-nic").arg("none")
			.arg("-nographic")
			.stdout(Stdio::piped())
			.spawn()?;
		qemu_process = qemu;
	} else {
		panic!("don't know how to run QEMU for current target platform")
	}

	let (mut qmp_stream, _) = qmp_sock.accept()?;
	qmp_stream.set_nodelay(true)?;
	qmp_stream.write_all(b"{\"execute\": \"qmp_capabilities\"}\n")?;

	let (serial_stream, _) = serial_sock.accept()?;
	serial_stream.set_nodelay(true)?;

	let (ready_send, ready_recv) = mpsc::sync_channel(0);
	let qemu_stdout = qemu_process.stdout.take().unwrap();
	thread::spawn(move || {
		let qemu_stdout = BufReader::new(qemu_stdout);
		let stdout = io::stdout();
		let mut ready = false;
		for line in qemu_stdout.split(b'\n') {
			let line = line.unwrap();
			let mut stdout_handle = stdout.lock();
			stdout_handle.write_all(&line).unwrap();
			stdout_handle.write_all(b"\n").unwrap();
			if !ready && line.starts_with(b"[rust-fuse] TEST_RUNNER_READY") {
				ready_send.send(()).unwrap();
				ready = true;
			}
		}
	});
	ready_recv.recv().unwrap();

	Ok(Qemu {
		process: qemu_process,
		qmp_stream: qmp_stream,
		serial_stream: serial_stream,
	})
}

fn run_test(testcase: &str) {
	println!(
		"[{:?}] ----- [test {}] -----",
		time::SystemTime::now(),
		testcase
	);

	let qemu = get_qemu();
	let serial_stream = &mut qemu.serial_stream;
	let mut testcase_path = std::path::PathBuf::new();
	testcase_path.push("fuse/tests/testcases");
	testcase_path.push(testcase);

	let mut client_path = testcase_path.clone();
	client_path.push("test_client");
	let mut client_file = fs::File::open(client_path).unwrap();
	frame_io::send_file(serial_stream, &mut client_file).unwrap();

	let mut server_path = testcase_path.clone();
	server_path.push("test_server");
	let mut server_file = fs::File::open(server_path).unwrap();
	frame_io::send_file(serial_stream, &mut server_file).unwrap();

	let client_stdout = frame_io::recv_frame(serial_stream).unwrap();
	let client_stderr = frame_io::recv_frame(serial_stream).unwrap();
	let server_stdout = frame_io::recv_frame(serial_stream).unwrap();
	let server_stderr = frame_io::recv_frame(serial_stream).unwrap();

	if !client_stderr.is_empty() {
		println!("vvvvvvvvvv [{} client_stderr] vvvvvvvvvv", testcase);
		io::stdout().write_all(&client_stderr).unwrap();
		println!("^^^^^^^^^^ [{} client_stderr] ^^^^^^^^^^", testcase);
	}
	if !server_stderr.is_empty() {
		println!("vvvvvvvvvv [{} server_stderr] vvvvvvvvvv", testcase);
		io::stdout().write_all(&server_stderr).unwrap();
		println!("^^^^^^^^^^ [{} server_stderr] ^^^^^^^^^^", testcase);
	}

	let mut client_stdout_path = testcase_path.clone();
	client_stdout_path.push("client_stdout.txt");
	let expected_client_stdout = fs::read(client_stdout_path).unwrap();

	let mut server_stdout_path = testcase_path.clone();
	server_stdout_path.push("server_stdout.txt");
	let expected_server_stdout = fs::read(server_stdout_path).unwrap();

	let mut failed = false;
	if let Some(diff) = diff_utf8(&expected_client_stdout, &client_stdout) {
		failed = true;
		println!("client_stdout mismatch:\n{}", diff);
	}
	if let Some(diff) = diff_utf8(&expected_server_stdout, &server_stdout) {
		failed = true;
		println!("server_stdout mismatch:\n{}", diff);
	}
	if failed {
		panic!("FAILED: {}", testcase);
	}
}

fn diff_utf8(want_utf8: &[u8], got_utf8: &[u8]) -> Option<String> {
	let want = std::str::from_utf8(want_utf8).unwrap();
	let got = std::str::from_utf8(got_utf8).unwrap();

	let mut out = String::new();
	let mut ok = true;
	for result in diff::lines(want, got) {
		match result {
			diff::Result::Left(l) => {
				ok = false;
				out.push_str("- ");
				out.push_str(l);
				out.push('\n');
			},
			diff::Result::Both(l, _) => {
				out.push_str("  ");
				out.push_str(l);
				out.push('\n');
			},
			diff::Result::Right(r) => {
				ok = false;
				out.push_str("+ ");
				out.push_str(r);
				out.push('\n');
			},
		}
	}

	if ok {
		return None;
	}
	Some(out)
}
