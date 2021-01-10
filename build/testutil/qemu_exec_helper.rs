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

use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::OpenOptionsExt;
use std::path;
use std::process::{Child, Command, ExitStatus};
use std::thread;

fn main() {
	match env::var_os("TEST_SRCDIR") {
		Some(_) => main_host(),
		None => main_guest(),
	};
}

fn main_host() {
	let mut argv = std::env::args();
	argv.next();

	let mut json_command = json::JsonValue::new_array();
	let mut test_binary: Option<String> = None;
	for arg in argv {
		if json_command.len() == 0 {
			test_binary = Some(arg.clone());
		}
		json_command.push(json::JsonValue::String(arg)).unwrap();
	}
	let test_binary = test_binary.unwrap();

	let json_env = json::JsonValue::new_object();

	let mut json_files = json::JsonValue::new_array();
	{
		let mut json_file = json::JsonValue::new_object();
		let meta = fs::metadata(&test_binary).unwrap();
		json_file.insert("path", test_binary.clone()).unwrap();
		json_file.insert("size", format!("{}", meta.len())).unwrap();

		json_files.push(json_file).unwrap();
	}

	let mut manifest = json::JsonValue::new_object();
	manifest.insert("command", json_command).unwrap();
	manifest.insert("env", json_env).unwrap();
	manifest.insert("files", json_files).unwrap();

	let mut manifest_buf = Cursor::new(Vec::new());
	manifest.write(&mut manifest_buf).unwrap();
	let manifest_json = manifest_buf.into_inner();

	let mut qemu = Qemu::new().unwrap();
	qemu.wait_ready().unwrap();

	send_frame(&mut qemu.control_stream, &manifest_json).unwrap();

	{
		let mut file = fs::File::open(&test_binary).unwrap();
		send_file(&mut qemu.control_stream, &mut file).unwrap();
		std::mem::drop(file);
	}

	let result_json = recv_frame(&mut qemu.control_stream).unwrap();
	let result =
		json::parse(std::str::from_utf8(&result_json).unwrap()).unwrap();

	println!("{}", result.pretty(4));

	qemu.quit().unwrap();
	std::process::exit(result["code"].as_i32().unwrap());
}

fn main_guest() {
	let mut virtio_control = fs::OpenOptions::new()
		.read(true)
		.write(true)
		.open("/dev/vport0p1")
		.unwrap();
	let virtio_stdout = fs::OpenOptions::new()
		.write(true)
		.open("/dev/vport0p2")
		.unwrap();
	let virtio_stderr = fs::OpenOptions::new()
		.write(true)
		.open("/dev/vport0p3")
		.unwrap();

	send_frame(&mut virtio_control, b"").unwrap();
	let manifest_json = recv_frame(&mut virtio_control).unwrap();
	let mut manifest =
		json::parse(std::str::from_utf8(&manifest_json).unwrap()).unwrap();

	std::env::set_current_dir("/rust-fuse/test_sandbox").unwrap();

	for file_json in manifest["files"].members() {
		let file_path =
			path::PathBuf::from(file_json["path"].as_str().unwrap());
		if let Some(parent) = file_path.parent() {
			fs::DirBuilder::new()
				.recursive(true)
				.create(parent)
				.unwrap();
		}
		let mut file = fs::OpenOptions::new()
			.write(true)
			.create_new(true)
			.mode(0o700)
			.open(&file_path)
			.unwrap();
		recv_file(&mut virtio_control, &mut file).unwrap();
		file.sync_all().unwrap();
		std::mem::drop(file);
	}

	let command_json = &mut manifest["command"].members();
	let command_path =
		format!("./{}", command_json.next().unwrap().as_str().unwrap());
	let mut test_command = Command::new(command_path);
	test_command.stdout(virtio_stdout);
	test_command.stderr(virtio_stderr);
	command_json.for_each(|arg| {
		let arg: &str = arg.as_str().unwrap();
		test_command.arg(arg);
	});

	let env_json = &mut manifest["env"];
	test_command.env_clear();
	env_json.entries().for_each(|(key, value)| {
		let value: &str = value.as_str().unwrap();
		test_command.env(key, value);
	});

	let test_status = test_command.status().unwrap();

	let mut result = json::JsonValue::new_object();
	result
		.insert("code", test_status.code().unwrap_or(1))
		.unwrap();
	let mut result_buf = Cursor::new(Vec::new());
	result.write(&mut result_buf).unwrap();
	let result_json = result_buf.into_inner();

	send_frame(&mut virtio_control, &result_json).unwrap();
}

struct Qemu {
	process: Child,
	qmp_stream: TcpStream,
	control_stream: TcpStream,
}

impl Qemu {
	fn new() -> io::Result<Qemu> {
		let qmp_sock = TcpListener::bind("127.0.0.1:0")?;
		let qmp_port = qmp_sock.local_addr()?.port();

		let control_sock = TcpListener::bind("127.0.0.1:0")?;
		let control_port = control_sock.local_addr()?.port();

		let stdout_sock = TcpListener::bind("127.0.0.1:0")?;
		let stdout_port = stdout_sock.local_addr()?.port();

		let stderr_sock = TcpListener::bind("127.0.0.1:0")?;
		let stderr_port = stderr_sock.local_addr()?.port();

		let qemu_process: Child;
		if let Ok(_) = fs::metadata("external/linux_kernel/arch/x86_64") {
			#[rustfmt::skip]
			let qemu = Command::new("qemu-system-x86_64")
				.arg("-m").arg("512M")
				.arg("-kernel").arg("external/linux_kernel/arch/x86_64/boot/bzImage")
				.arg("-initrd").arg("build/testutil/initrd.cpio.gz")
				.arg("-serial").arg("stdio")
				.arg("-chardev").arg(format!("socket,id=virtio-control,host=127.0.0.1,port={}", control_port))
				.arg("-chardev").arg(format!("socket,id=virtio-stdout,host=127.0.0.1,port={}", stdout_port))
				.arg("-chardev").arg(format!("socket,id=virtio-stderr,host=127.0.0.1,port={}", stderr_port))
				.arg("-device").arg("virtio-serial")
				.arg("-device").arg("virtserialport,chardev=virtio-control,id=test0,nr=1")
				.arg("-device").arg("virtserialport,chardev=virtio-stdout,id=test1,nr=2")
				.arg("-device").arg("virtserialport,chardev=virtio-stderr,id=test2,nr=3")
				.arg("-qmp").arg(format!("tcp:127.0.0.1:{},nodelay", qmp_port))
				.arg("-append").arg("console=ttyS0 rdinit=/bin/init")
				.arg("-nic").arg("none")
				.arg("-nographic")
				.spawn()?;
			qemu_process = qemu;
		} else {
			panic!("don't know how to run QEMU for current target platform")
		}

		let (mut qmp_stream, _) = qmp_sock.accept()?;
		qmp_stream.set_nodelay(true)?;
		qmp_stream.write_all(b"{\"execute\": \"qmp_capabilities\"}\n")?;

		let (control_stream, _) = control_sock.accept()?;
		control_stream.set_nodelay(true)?;

		let (stdout_stream, _) = stdout_sock.accept()?;
		stdout_stream.set_nodelay(true)?;

		let (stderr_stream, _) = stderr_sock.accept()?;
		stderr_stream.set_nodelay(true)?;

		thread::spawn(move || {
			let test_stdout = BufReader::new(stdout_stream);
			let stdout = io::stdout();
			for line in test_stdout.split(b'\n') {
				let line = line.unwrap();
				let mut stdout_handle = stdout.lock();
				stdout_handle.write_all(&line).unwrap();
				stdout_handle.write_all(b"\n").unwrap();
			}
		});

		thread::spawn(move || {
			let test_stderr = BufReader::new(stderr_stream);
			let stderr = io::stderr();
			for line in test_stderr.split(b'\n') {
				let line = line.unwrap();
				let mut stderr_handle = stderr.lock();
				stderr_handle.write_all(&line).unwrap();
				stderr_handle.write_all(b"\n").unwrap();
			}
		});

		Ok(Qemu {
			process: qemu_process,
			qmp_stream: qmp_stream,
			control_stream: control_stream,
		})
	}

	fn wait_ready(&mut self) -> io::Result<()> {
		recv_frame(&mut self.control_stream)?;
		Ok(())
	}

	fn quit(&mut self) -> io::Result<ExitStatus> {
		self.qmp_stream
			.write_all(b"{\"execute\": \"quit\"}\n")
			.unwrap();
		{
			let mut buf = Vec::new();
			let _ = self.qmp_stream.read_to_end(&mut buf);
		}

		self.process.wait()
	}
}

fn send_frame(w: &mut impl Write, buf: &[u8]) -> io::Result<()> {
	let len_buf = (buf.len() as u64).to_be_bytes();
	w.write_all(&len_buf)?;
	w.write_all(buf)
}

fn send_file(w: &mut impl Write, file: &mut fs::File) -> io::Result<()> {
	let file_len: u64 = file.metadata()?.len();
	let file_len_buf = file_len.to_be_bytes();
	w.write_all(&file_len_buf)?;
	io::copy(file, w)?;
	Ok(())
}

fn recv_frame(r: &mut impl Read) -> io::Result<Vec<u8>> {
	let mut len_buf = [0; 8];
	r.read_exact(&mut len_buf)?;
	let len = u64::from_be_bytes(len_buf);
	let mut buf = Vec::new();
	if len > 0 {
		r.take(len).read_to_end(&mut buf)?;
	}
	Ok(buf)
}

fn recv_file(r: &mut impl Read, file: &mut fs::File) -> io::Result<()> {
	let mut len_buf = [0; 8];
	r.read_exact(&mut len_buf)?;
	let len = u64::from_be_bytes(len_buf);
	if len > 0 {
		io::copy(&mut r.take(len), file)?;
	}
	Ok(())
}
