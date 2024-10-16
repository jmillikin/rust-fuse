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

use std::num::NonZeroU64;
use std::os::unix::ffi::OsStrExt;
use std::sync::mpsc;
use std::{ffi, panic};

use fuse::server;
use fuse::server::FuseRequest;

use interop_testutil::{
	diff_str,
	fuse_interop_test,
	path_cstr,
	OsError,
};

struct TestFS {
	requests: mpsc::Sender<String>,
}

struct TestHandlers<'a, S> {
	fs: &'a TestFS,
	conn: &'a server::FuseConnection<S>,
}

impl interop_testutil::TestFS for TestFS {
	fn dispatch_request(
		&self,
		conn: &server::FuseConnection<interop_testutil::DevFuse>,
		request: FuseRequest<'_>,
	) {
		use fuse::server::FuseHandlers;
		(TestHandlers{fs: self, conn}).dispatch(request);
	}
}

impl<'a, S> server::FuseHandlers for TestHandlers<'a, S>
where
	S: server::FuseSocket,
	S::Error: core::fmt::Debug,
{
	fn unimplemented(&self, request: FuseRequest<'_>) {
		self.conn.reply(request.id()).err(OsError::UNIMPLEMENTED).unwrap();
	}

	fn lookup(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::LookupRequest::try_from(request).unwrap();

		if !request.parent_id().is_root() {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}
		if request.name() != "readdir.d" {
			return send_reply.err(OsError::NOT_FOUND).unwrap();
		}

		let mut attr = fuse::NodeAttr::new(fuse::NodeId::new(2).unwrap());
		attr.set_mode(fuse::FileMode::S_IFDIR | 0o755);
		attr.set_link_count(2);

		let mut entry = fuse::Entry::new(attr);
		entry.set_cache_timeout(std::time::Duration::from_secs(60));

		send_reply.ok(&entry).unwrap();
	}

	fn opendir(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let mut reply = fuse::kernel::fuse_open_out::new();
		reply.fh = 12345;
		send_reply.ok(&reply).unwrap();
	}

	fn readdir(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		let request = server::ReaddirRequest::try_from(request).unwrap();
		self.fs.requests.send(format!("{:#?}", request)).unwrap();

		let mut offset: u64 = match request.offset() {
			Some(x) => x.into(),
			None => 0,
		};

		let mut buf = vec![0u8; request.size() as usize];
		let mut entries = server::ReaddirEntriesWriter::new(&mut buf);

		if offset == 0 {
			offset += 1;
			let mut entry = server::ReaddirEntry::new(
				fuse::NodeId::new(10).unwrap(),
				fuse::NodeName::new("entry_a").unwrap(),
				NonZeroU64::new(offset).unwrap(),
			);
			entry.set_file_type(fuse::FileType::Regular);
			entries.try_push(&entry).unwrap();
		}
		if offset == 1 {
			offset += 1;
			let mut entry = server::ReaddirEntry::new(
				fuse::NodeId::new(11).unwrap(),
				fuse::NodeName::new("entry_b").unwrap(),
				NonZeroU64::new(offset).unwrap(),
			);
			entry.set_file_type(fuse::FileType::Symlink);
			entries.try_push(&entry).unwrap();

			return send_reply.ok(&entries.into_entries()).unwrap();
		}

		if offset == 2 {
			offset += 1;
			let mut entry = server::ReaddirEntry::new(
				fuse::NodeId::new(12).unwrap(),
				fuse::NodeName::new("entry_c").unwrap(),
				NonZeroU64::new(offset).unwrap(),
			);
			entry.set_file_type(fuse::FileType::Directory);
			entries.try_push(&entry).unwrap();
		}

		send_reply.ok(&entries.into_entries()).unwrap();
	}

	fn releasedir(&self, request: FuseRequest<'_>) {
		let send_reply = self.conn.reply(request.id());
		send_reply.ok_empty().unwrap();
	}
}

fn readdir_test(
	test_fn: impl FnOnce(&std::path::Path) + panic::UnwindSafe,
) -> Vec<String> {
	let (request_send, request_recv) = mpsc::channel();
	let fs = TestFS {
		requests: request_send,
	};
	fuse_interop_test(fs, test_fn);
	request_recv.iter().collect()
}

#[test]
fn readdir() {
	let requests = readdir_test(|root| {
		let path = path_cstr(root.join("readdir.d"));

		let dir_p = unsafe { libc::opendir(path.as_ptr()) };
		assert!(!dir_p.is_null());

		let next_dirent = |expect| {
			let entry = unsafe { libc::readdir(dir_p) };
			assert!(!entry.is_null());
			if let Some(diff) = diff_dirent(expect, unsafe { &*entry }) {
				println!("{}", diff);
				assert!(false);
			}
		};

		next_dirent(dirent {
			d_ino: 10,
			d_off: 1,
			d_reclen: 32,
			d_type: libc::DT_REG,
			#[cfg(target_os = "freebsd")]
			d_namlen: 7,
			d_name: dirent_name_new(b"entry_a"),
		});
		let pos = unsafe { libc::telldir(dir_p) };
		next_dirent(dirent {
			d_ino: 11,
			d_off: 2,
			d_reclen: 32,
			d_type: libc::DT_LNK,
			#[cfg(target_os = "freebsd")]
			d_namlen: 7,
			d_name: dirent_name_new(b"entry_b"),
		});
		unsafe {
			libc::seekdir(dir_p, pos)
		};
		next_dirent(dirent {
			d_ino: 11,
			d_off: 2,
			d_reclen: 32,
			d_type: libc::DT_LNK,
			#[cfg(target_os = "freebsd")]
			d_namlen: 7,
			d_name: dirent_name_new(b"entry_b"),
		});
		next_dirent(dirent {
			d_ino: 12,
			d_off: 3,
			d_reclen: 32,
			d_type: libc::DT_DIR,
			#[cfg(target_os = "freebsd")]
			d_namlen: 7,
			d_name: dirent_name_new(b"entry_c"),
		});

		unsafe {
			libc::closedir(dir_p)
		};
	});

	#[cfg(target_os = "linux")]
	{
		assert_eq!(requests.len(), 3);

		let expect = r#"ReaddirRequest {
    node_id: 2,
    size: 4096,
    offset: None,
    handle: 12345,
    open_flags: 0x00018000,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"ReaddirRequest {
    node_id: 2,
    size: 4096,
    offset: Some(1),
    handle: 12345,
    open_flags: 0x00018000,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"ReaddirRequest {
    node_id: 2,
    size: 4096,
    offset: Some(2),
    handle: 12345,
    open_flags: 0x00018000,
}"#;
		if let Some(diff) = diff_str(expect, &requests[2]) {
			println!("{}", diff);
			assert!(false);
		}
	}

	#[cfg(target_os = "freebsd")]
	{
		assert_eq!(requests.len(), 2);

		let expect = r#"ReaddirRequest {
    node_id: 2,
    size: 4096,
    offset: None,
    handle: 12345,
    open_flags: 0x00000000,
}"#;
		if let Some(diff) = diff_str(expect, &requests[0]) {
			println!("{}", diff);
			assert!(false);
		}

		let expect = r#"ReaddirRequest {
    node_id: 2,
    size: 4096,
    offset: Some(2),
    handle: 12345,
    open_flags: 0x00000000,
}"#;
		if let Some(diff) = diff_str(expect, &requests[1]) {
			println!("{}", diff);
			assert!(false);
		}
	}
}

#[allow(non_camel_case_types)]
struct dirent<'a> {
	d_ino: libc::ino_t,
	d_off: libc::off_t,
	d_reclen: libc::c_ushort,
	d_type: u8,
	#[cfg(target_os = "freebsd")]
	d_namlen: u16,
	d_name: &'a ffi::OsStr,
}

impl std::fmt::Debug for dirent<'_> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		let d_type = dirent_type_name(self);
		let mut s = fmt.debug_struct("dirent");
		#[cfg(target_os = "linux")]
		s.field("d_ino", &self.d_ino);
		#[cfg(target_os = "freebsd")]
		s.field("d_fileno", &self.d_ino);
		s.field("d_off", &self.d_off);
		s.field("d_reclen", &self.d_reclen);
		s.field("d_type", &format_args!("{}", &d_type));
		#[cfg(target_os = "freebsd")]
		s.field("d_namlen", &self.d_namlen);
		s.field("d_name", &self.d_name);
		s.finish()
	}
}

#[allow(unused_mut)]
fn diff_dirent(mut expect: dirent, got: &libc::dirent) -> Option<String> {
	let expect = format!("{:#?}", &expect);
	let got = format!(
		"{:#?}",
		dirent {
			#[cfg(target_os = "linux")]
			d_ino: got.d_ino,
			#[cfg(target_os = "freebsd")]
			d_ino: got.d_fileno,
			d_off: got.d_off,
			d_reclen: got.d_reclen,
			d_type: got.d_type,
			#[cfg(target_os = "freebsd")]
			d_namlen: got.d_namlen,
			d_name: dirent_name(got),
		}
	);
	diff_str(&expect, &got)
}

fn dirent_name(dirent: &libc::dirent) -> &ffi::OsStr {
	let bytes: &[u8] = unsafe { std::mem::transmute(&dirent.d_name as &[i8]) };
	for (ii, byte) in bytes.iter().enumerate() {
		if *byte == 0 {
			let (name, _) = bytes.split_at(ii);
			return ffi::OsStr::from_bytes(name);
		}
	}
	panic!("no NUL in dirent d_name")
}

fn dirent_name_new(name: &[u8]) -> &ffi::OsStr {
	ffi::OsStr::from_bytes(name)
}

fn dirent_type_name(dirent: &dirent) -> String {
	match dirent.d_type {
		libc::DT_BLK => "DT_BLK".to_string(),
		libc::DT_CHR => "DT_CHR".to_string(),
		libc::DT_DIR => "DT_DIR".to_string(),
		libc::DT_FIFO => "DT_FIFO".to_string(),
		libc::DT_LNK => "DT_LNK".to_string(),
		libc::DT_REG => "DT_REG".to_string(),
		libc::DT_SOCK => "DT_SOCK".to_string(),
		libc::DT_UNKNOWN => "DT_UNKNOWN".to_string(),
		x => format!("{}", x),
	}
}
