// Copyright 2024 John Millikin and the rust-fuse contributors.
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

use std::alloc::Layout;
use std::sync::mpsc;

use fuse::kernel::FUSE_MIN_READ_BUFFER;
use fuse::io::{
	AlignedSlice,
	AlignedSliceMut,
	AsAlignedSlice,
	AsAlignedSliceMut,
};
use fuse::server::{
	cuse_rpc,
	fuse_rpc,
	CuseConnection,
	FuseConnection,
	ServerError,
};
use fuse::server::io::{CuseSocket, FuseSocket};

fn server_threads() -> usize {
	// Use `thread::available_parallelism()` to estimate how many hardware
	// threads might be available. This number is clamped to 16 to avoid
	// allocating an unreasonable amount of memory on larger machines.
	//
	// It's expected that this estimate won't work for all possible servers,
	// either because it's too small (in a server doing lots of slow remote IO)
	// or too large (in a constrained environment). Since the `serve()` function
	// uses only public API, servers with special requirements can write their
	// own version with appropriate threadpool sizing.
	const MAX_THREADS: usize = 16;
	core::cmp::min(
		std::thread::available_parallelism().map_or(1, |n| n.get()),
		MAX_THREADS,
	)
}

/// Serve FUSE requests in a multi-threaded loop.
///
/// This function spawns worker threads to process FUSE requests from the
/// given channel. The returned [`mpsc::Receiver`] can be used to receive
/// server errors from the worker threads, or dropped to run without error
/// reporting.
///
/// The worker threads will terminate if:
/// * An I/O error is reported by the socket.
/// * The connection is closed, such as by the user unmounting the filesystem
///   with `fusermount -u`.
///
/// # Panics
///
/// Panics on memory allocation failure. This function allocates
/// [`conn.recv_buf_len()`] bytes per worker thread, and also calls standard
/// library APIs such as [`Vec::with_capacity`] that panic on OOM.
///
/// [`conn.recv_buf_len()`]: FuseConnection::recv_buf_len
pub fn serve_fuse<S, H>(
	conn: &FuseConnection<S>,
	handlers: &H,
) -> mpsc::Receiver<ServerError<S::Error>>
where
	S: FuseSocket + Send + Sync,
	S::Error: Send,
	H: fuse_rpc::Handlers<S> + Send + Sync,
{
	// Pre-allocate receive buffers so that an allocation failure will happen
	// before any server threads get spawned.
	let num_threads = server_threads();
	let mut recv_bufs = Vec::with_capacity(num_threads);
	let recv_buf_len = conn.recv_buf_len();
	for _ii in 0..num_threads {
		recv_bufs.push(AlignedBuf::with_capacity(recv_buf_len));
	}

	let (err_sender, err_receiver) = mpsc::sync_channel(num_threads);
	std::thread::scope(|s| {
		for _ii in 0..num_threads {
			let err_sender = err_sender.clone();
			let mut buf = recv_bufs.remove(recv_bufs.len() - 1);
			s.spawn(move || {
				while let Err(err) = fuse_rpc::serve_local(conn, handlers, &mut buf) {
					let fatal = is_fatal_error(&err);
					let _ = err_sender.send(err);
					if fatal {
						return;
					}
				}
			});
		}
	});

	err_receiver
}

/// Serve CUSE requests in a multi-threaded loop.
///
/// This function spawns worker threads to process CUSE requests from the
/// given channel. The returned [`mpsc::Receiver`] can be used to receive
/// server errors from the worker threads, or dropped to run without error
/// reporting.
///
/// The worker threads will terminate if an I/O error is reported by the
/// socket.
///
/// # Panics
///
/// Panics on memory allocation failure. This function allocates
/// [`conn.recv_buf_len()`] bytes per worker thread, and also calls standard
/// library APIs such as [`Vec::with_capacity`] that panic on OOM.
///
/// [`conn.recv_buf_len()`]: CuseConnection::recv_buf_len
pub fn serve_cuse<S, H>(
	conn: &CuseConnection<S>,
	handlers: &H,
) -> mpsc::Receiver<ServerError<S::Error>>
where
	S: CuseSocket + Send + Sync,
	S::Error: Send,
	H: cuse_rpc::Handlers<S> + Send + Sync,
{
	// Pre-allocate receive buffers so that an allocation failure will happen
	// before any server threads get spawned.
	let num_threads = server_threads();
	let mut recv_bufs = Vec::with_capacity(num_threads);
	let recv_buf_len = conn.recv_buf_len();
	for _ii in 0..num_threads {
		recv_bufs.push(AlignedBuf::with_capacity(recv_buf_len));
	}

	let (err_sender, err_receiver) = mpsc::sync_channel(num_threads);
	std::thread::scope(|s| {
		for _ii in 0..num_threads {
			let err_sender = err_sender.clone();
			let mut buf = recv_bufs.remove(recv_bufs.len() - 1);
			s.spawn(move || {
				while let Err(err) = cuse_rpc::serve_local(conn, handlers, &mut buf) {
					let fatal = is_fatal_error(&err);
					let _ = err_sender.send(err);
					if fatal {
						return;
					}
				}
			});
		}
	});

	err_receiver
}

fn is_fatal_error<E>(err: &ServerError<E>) -> bool {
	match err {
		ServerError::RequestError(_) => false,
		_ => true,
	}
}

/// A heap-allocated buffer with appropriate alignment for FUSE messages.
pub struct AlignedBuf {
	ptr: core::ptr::NonNull<u8>,
	len: usize,
}

unsafe impl Send for AlignedBuf {}

unsafe impl Sync for AlignedBuf {}

impl Drop for AlignedBuf {
	fn drop(&mut self) {
		unsafe {
			let layout = Layout::from_size_align_unchecked(self.len, 8);
			std::alloc::dealloc(self.ptr.as_ptr(), layout)
		}
	}
}

impl AlignedBuf {
	/// Allocates a new `AlignedBuf` with capacity [`FUSE_MIN_READ_BUFFER`].
	#[must_use]
	pub fn new() -> AlignedBuf {
		Self::with_capacity(FUSE_MIN_READ_BUFFER)
	}

	/// Allocates a new `AlignedBuf` with at least the specified capacity.
	#[must_use]
	pub fn with_capacity(capacity: usize) -> AlignedBuf {
		let capacity = core::cmp::max(capacity, FUSE_MIN_READ_BUFFER);
		let layout = Layout::from_size_align(capacity, 8).unwrap();
		let ptr = unsafe { std::alloc::alloc(layout) };
		match core::ptr::NonNull::new(ptr) {
			Some(ptr) => AlignedBuf { ptr, len: capacity },
			None => std::alloc::handle_alloc_error(layout),
		}
	}

	/// Borrows this `AlignedBuf` as a byte slice.
	#[inline]
	#[must_use]
	pub fn as_slice(&self) -> &[u8] {
		unsafe {
			core::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
		}
	}

	/// Borrows this `AlignedBuf` as a mutable byte slice.
	#[inline]
	#[must_use]
	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		unsafe {
			core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
		}
	}
}

impl AsAlignedSlice for AlignedBuf {
	#[inline]
	fn as_aligned_slice(&self) -> AlignedSlice {
		unsafe {
			AlignedSlice::new_unchecked(self.as_slice())
		}
	}
}

impl AsAlignedSliceMut for AlignedBuf {
	#[inline]
	fn as_aligned_slice_mut(&mut self) -> AlignedSliceMut {
		unsafe {
			AlignedSliceMut::new_unchecked(self.as_mut_slice())
		}
	}
}
