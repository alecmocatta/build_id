//! Obtain a [Uuid] uniquely representing the build of the current binary.
//!
//! This is intended to be used to check that different processes are indeed
//! invocations of identically laid out binaries.
//!
//! As such:
//! * It is guaranteed to be identical within multiple invocations of the same
//! binary.
//! * It is guaranteed to be different across binaries with different code or
//! data segments or layout.
//! * Equality is unspecified if the binaries have identical code and data
//! segments and layout but differ immaterially (e.g. if a timestamp is included
//! in the binary at compile time).
//!
//! # Examples
//!
//! ```
//! # let remote_build_id = build_id::get();
//! let local_build_id = build_id::get();
//! if local_build_id == remote_build_id {
//! 	println!("We're running the same binary as remote!");
//! } else {
//! 	println!("We're running a different binary to remote");
//! }
//! ```
//!
//! # Note
//!
//! This looks first for linker-inserted build ID / binary UUIDs (i.e.
//! `.note.gnu.build-id` on Linux; `LC_UUID` in Mach-O; etc), falling back to
//! hashing the whole binary.

#![doc(html_root_url = "https://docs.rs/build_id/0.1.1")]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	trivial_numeric_casts,
	unused_extern_crates,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)] // from https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
#![allow(clippy::stutter)]

extern crate byteorder;
extern crate proc_self;
extern crate twox_hash;
extern crate uuid;

use std::{hash::Hasher, io, sync};
use uuid::Uuid;

static mut BUILD_ID: Uuid = Uuid::nil();
static INIT: sync::Once = sync::ONCE_INIT;

/// Returns a [Uuid] uniquely representing the build of the current binary.
///
/// This is intended to be used to check that different processes are indeed
/// invocations of identically laid out binaries.
///
/// As such:
/// * It is guaranteed to be identical within multiple invocations of the same
/// binary.
/// * It is guaranteed to be different across binaries with different code or
/// data segments or layout.
/// * Equality is unspecified if the binaries have identical code and data
/// segments and layout but differ immaterially (e.g. if a timestamp is included
/// in the binary at compile time).
///
/// # Examples
///
/// ```
/// # let remote_build_id = build_id::get();
/// let local_build_id = build_id::get();
/// if local_build_id == remote_build_id {
/// 	println!("We're running the same binary as remote!");
/// } else {
/// 	println!("We're running a different binary to remote");
/// }
/// ```
///
/// # Note
///
/// This looks first for linker-inserted build ID / binary UUIDs (i.e.
/// `.note.gnu.build-id` on Linux; `LC_UUID` in Mach-O; etc), falling back to
/// hashing the whole binary.
#[inline]
pub fn get() -> Uuid {
	unsafe {
		INIT.call_once(|| {
			BUILD_ID = calculate();
		});
		BUILD_ID
	}
}
fn calculate() -> Uuid {
	let mut hasher = twox_hash::XxHash::with_seed(0);

	// let a = |x:()|x;
	// let b = |x:u8|x;
	// hasher.write_u64(type_id(&a));
	// hasher.write_u64(type_id(&b));

	// LC_UUID https://opensource.apple.com/source/libsecurity_codesigning/libsecurity_codesigning-55037.6/lib/machorep.cpp https://stackoverflow.com/questions/10119700/how-to-get-mach-o-uuid-of-a-running-process
	// .note.gnu.build-id https://github.com/golang/go/issues/21564 https://github.com/golang/go/blob/178307c3a72a9da3d731fecf354630761d6b246c/src/cmd/go/internal/buildid/buildid.go
	let file = proc_self::exe().unwrap();
	let _ = io::copy(&mut &file, &mut HashWriter(&mut hasher)).unwrap();

	let mut bytes = [0; 16];
	<byteorder::NativeEndian as byteorder::ByteOrder>::write_u64(&mut bytes, hasher.finish());
	Uuid::from_random_bytes(bytes)
}

// fn type_id<T:'static>(_: &T) -> u64 {
// 	unsafe{intrinsics::type_id::<T>()}
// }

struct HashWriter<T: Hasher>(T);
impl<T: Hasher> io::Write for HashWriter<T> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.0.write(buf);
		Ok(buf.len())
	}
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
		self.write(buf).map(|_| ())
	}
	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn brute() {
		let x = super::calculate();
		for _ in 0..1000 {
			assert_eq!(x, super::calculate());
		}
	}
	#[test]
	fn get() {
		let x = super::calculate();
		assert_eq!(x, super::get());
		assert_eq!(x, super::get());
		assert_eq!(x, super::get());
	}
}
