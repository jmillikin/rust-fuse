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

#![allow(unused_imports)]

use std::convert::TryInto;
use std::env;
use std::ffi::OsString;
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

	qemu.control_stream.send(&manifest_json).unwrap();

	{
		let mut file = fs::File::open(&test_binary).unwrap();
		send_file(&mut qemu.control_stream, &mut file).unwrap();
		std::mem::drop(file);
	}

	let mut recv_buf = [0u8; PAYLOAD_MTU];
	let result_json = qemu.control_stream.recv(&mut recv_buf).unwrap();
	let result =
		json::parse(std::str::from_utf8(&result_json).unwrap()).unwrap();

	println!("{}", result.pretty(4));

	qemu.quit().unwrap();
	std::process::exit(result["code"].as_i32().unwrap());
}

#[cfg(any(target_os = "linux", target_os = "freebsd",))]
fn main_guest() {
	let mut virtio_control = GuestVirtioStream::new(1).unwrap();
	let virtio_stdout = GuestVirtioStream::new(2).unwrap().file;
	let virtio_stderr = GuestVirtioStream::new(3).unwrap().file;

	virtio_control.send(b"").unwrap();
	let mut recv_buf = [0u8; PAYLOAD_MTU];
	let manifest_json = virtio_control.recv(&mut recv_buf).unwrap();
	let mut manifest =
		json::parse(std::str::from_utf8(&manifest_json).unwrap()).unwrap();

	std::env::set_current_dir("/rust-fuse/test_sandbox").unwrap();

	println!("[GUEST] receiving test files ...");
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

	println!("[GUEST] running test ...");
	let test_status = test_command.status().unwrap();
	println!("[GUEST] test complete!");

	let mut result = json::JsonValue::new_object();
	result
		.insert("code", test_status.code().unwrap_or(1))
		.unwrap();
	let mut result_buf = Cursor::new(Vec::new());
	result.write(&mut result_buf).unwrap();
	let result_json = result_buf.into_inner();

	virtio_control.send(&result_json).unwrap();
}

#[cfg(not(any(
	target_os = "linux",
	target_os = "freebsd",
)))]
fn main_guest() {}

struct Qemu {
	process: Child,
	qmp_stream: TcpStream,
	control_stream: HostVirtioStream,
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

		let test_cpu = std::env::var("RUST_FUSE_TEST_CPU").unwrap();
		let test_os = std::env::var("RUST_FUSE_TEST_OS").unwrap();
		let test_rootfs = std::env::var("RUST_FUSE_TEST_ROOTFS").unwrap();

		let rootfs_path = path::Path::new(&test_rootfs);

		let qemu_common_args = |qemu: &mut Command| {
			qemu.arg("-m").arg("512M");
			qemu.arg("-serial").arg("stdio");
			qemu.arg("-chardev").arg(format!("socket,id=virtio-control,host=127.0.0.1,port={}", control_port));
			qemu.arg("-chardev").arg(format!("socket,id=virtio-stdout,host=127.0.0.1,port={}", stdout_port));
			qemu.arg("-chardev").arg(format!("socket,id=virtio-stderr,host=127.0.0.1,port={}", stderr_port));
			qemu.arg("-device").arg("virtio-serial");
			qemu.arg("-device").arg("virtserialport,chardev=virtio-control,id=test0,nr=1");
			qemu.arg("-device").arg("virtserialport,chardev=virtio-stdout,id=test1,nr=2");
			qemu.arg("-device").arg("virtserialport,chardev=virtio-stderr,id=test2,nr=3");
			qemu.arg("-qmp").arg(format!("tcp:127.0.0.1:{},nodelay", qmp_port));
			qemu.arg("-nic").arg("none");
			qemu.arg("-nographic");
		};

		let mut qemu = match (&test_os as &str, &test_cpu as &str) {
			("linux", "x86_64") => {
				let mut qemu = Command::new("qemu-system-x86_64");
				qemu_common_args(&mut qemu);
				qemu.arg("-kernel").arg(rootfs_path.join("boot").join("bzImage"));
				qemu.arg("-initrd").arg(rootfs_path.join("boot").join("initrd.cpio.gz"));
				qemu.arg("-append").arg("console=ttyS0 rdinit=/bin/init");
				qemu
			},
			("freebsd", "x86_64") => {
				let mut qemu = Command::new("qemu-system-x86_64");
				qemu_common_args(&mut qemu);
				qemu.arg("-kernel").arg(rootfs_path.join("boot").join("loader_simp.efi"));
				qemu.arg("-drive").arg("if=pflash,format=raw,unit=0,file=external/qemu_v5.2.0/pc-bios/edk2-x86_64-code.fd,readonly=on");

				let mut drive_arg = OsString::from("format=raw,file=fat:rw:");
				drive_arg.push(rootfs_path);
				qemu.arg("-drive").arg(drive_arg);

				qemu.arg("-append").arg("rootdev=disk0p1 currdev=disk0p1 autoboot_delay=-1 vfs.root.mountfrom=msdosfs:/dev/ada0s1 init_path=/sbin/init");
				qemu
			}
			_ => panic!("don't know how to run QEMU for current target platform"),
		};

		let qemu_process = qemu.spawn()?;

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
			control_stream: HostVirtioStream::new(control_stream),
		})
	}

	fn wait_ready(&mut self) -> io::Result<()> {
		let mut buf = [0u8; PAYLOAD_MTU];
		self.control_stream.recv(&mut buf)?;
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

const PACKET_MTU: usize = 22000;
const PAYLOAD_MTU: usize = 16500;

trait VirtioStream {
	fn send(&mut self, buf: &[u8]) -> io::Result<()>;
	fn recv<'a>(
		&mut self,
		buf: &'a mut [u8; PAYLOAD_MTU],
	) -> io::Result<&'a [u8]>;
}

struct HostVirtioStream {
	stream: TcpStream,
}

impl HostVirtioStream {
	fn new(stream: TcpStream) -> Self {
		Self { stream }
	}
}

impl VirtioStream for HostVirtioStream {
	fn send(&mut self, buf: &[u8]) -> io::Result<()> {
		send_packet(&mut self.stream, buf)
	}

	fn recv<'a>(
		&mut self,
		buf: &'a mut [u8; PAYLOAD_MTU],
	) -> io::Result<&'a [u8]> {
		recv_packet(&mut self.stream, buf)
	}
}

struct GuestVirtioStream {
	file: fs::File,
}

impl GuestVirtioStream {
	#[cfg(target_os = "linux")]
	fn new(virtio_port: u8) -> io::Result<Self> {
		let virtio_device = format!("/dev/vport0p{}", virtio_port);
		let file = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open(&virtio_device)?;
		Ok(Self { file })
	}

	#[cfg(target_os = "freebsd")]
	fn new(virtio_port: u8) -> io::Result<Self> {
		let virtio_device = format!("/dev/ttyV0.{}", virtio_port);

		let file = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open(&virtio_device)?;

		Command::new("/rescue/rescue")
			.arg("stty")
			.arg("-f")
			.arg(&virtio_device)
			.arg("raw")
			.arg("speed")
			.arg("115200")
			.output()?;

		Ok(Self { file })
	}
}

impl VirtioStream for GuestVirtioStream {
	fn send(&mut self, buf: &[u8]) -> io::Result<()> {
		send_packet(&mut self.file, buf)
	}

	fn recv<'a>(
		&mut self,
		buf: &'a mut [u8; PAYLOAD_MTU],
	) -> io::Result<&'a [u8]> {
		recv_packet(&mut self.file, buf)
	}
}

fn send_packet(w: &mut impl Write, payload: &[u8]) -> io::Result<()> {
	let mut header = [0u8; 9];

	let payload_len: [u8; 4] = (payload.len() as u32).to_be_bytes();
	(&mut header[0..4]).copy_from_slice(&payload_len);

	let checksum = [0u8; 4]; // TODO
	(&mut header[4..8]).copy_from_slice(&checksum);

	let mut base64_header = [0u8; 13];
	base64::encode_config_slice(header, base64::STANDARD, &mut base64_header);
	base64_header[12] = b'\n';
	w.write_all(&base64_header)?;

	let mut base64_payload = [0u8; PACKET_MTU + 1];
	let base64_payload_len = base64::encode_config_slice(
		payload,
		base64::STANDARD,
		&mut base64_payload,
	);
	base64_payload[base64_payload_len] = b'\n';
	w.write_all(&base64_payload[..base64_payload_len + 1])?;

	Ok(())
}

fn recv_packet<'a>(
	r: &mut impl Read,
	buf: &'a mut [u8; PAYLOAD_MTU],
) -> io::Result<&'a [u8]> {
	let mut base64_header = [0u8; 13];
	let mut header = [0u8; 9];
	r.read_exact(&mut base64_header)?;
	base64::decode_config_slice(
		&base64_header[0..12],
		base64::STANDARD,
		&mut header,
	)
	.unwrap();

	let payload_len =
		u32::from_be_bytes(header[0..4].try_into().unwrap()) as usize;
	let checksum = u32::from_be_bytes(header[4..8].try_into().unwrap());
	let _checksum = checksum; // TODO

	let mut base64_payload = [0u8; PACKET_MTU + 1];
	let mut base64_payload_len = 4 * (payload_len / 3);
	if payload_len % 3 > 0 {
		base64_payload_len += 4;
	}

	r.read_exact(&mut base64_payload[..base64_payload_len + 1])?;
	let decoded_len = base64::decode_config_slice(
		&base64_payload[..base64_payload_len],
		base64::STANDARD,
		buf,
	)
	.unwrap();
	assert_eq!(payload_len, decoded_len);

	Ok(&buf[..payload_len])
}

fn send_file(
	sock: &mut impl VirtioStream,
	file: &mut fs::File,
) -> io::Result<()> {
	let file_len: u64 = file.metadata()?.len();
	let file_len_buf = file_len.to_be_bytes();
	sock.send(&file_len_buf)?;

	let mut buf = [0u8; PAYLOAD_MTU];
	loop {
		match file.read(&mut buf)? {
			0 => return Ok(()),
			len => {
				sock.send(&buf[..len])?;
				sock.recv(&mut buf)?; // ACK
			},
		};
	}
}

#[allow(dead_code)]
fn recv_file(
	sock: &mut impl VirtioStream,
	file: &mut fs::File,
) -> io::Result<()> {
	let mut buf = [0u8; PAYLOAD_MTU];
	sock.recv(&mut buf)?;
	let len = u64::from_be_bytes(buf[0..8].try_into().unwrap());
	if len == 0 {
		return Ok(());
	}

	let mut remaining = len;
	while remaining > 0 {
		let received = sock.recv(&mut buf)?;
		file.write_all(received)?;
		remaining -= received.len() as u64;

		sock.send(b"ACK")?;
	}
	Ok(())
}
