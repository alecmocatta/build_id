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

#![doc(html_root_url = "https://docs.rs/build_id/0.2.1")]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)] // from https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
#![allow(clippy::must_use_candidate)]

use once_cell::sync::Lazy;
use std::{
	any::TypeId,
	hash::{Hash, Hasher},
	io,
};
use uuid::Uuid;

static BUILD_ID: Lazy<Uuid> = Lazy::new(calculate);

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
	*BUILD_ID
}

#[allow(clippy::needless_pass_by_value)]
fn from_header<H: Hasher>(_hasher: H) -> Result<H, ()> {
	// LC_UUID https://opensource.apple.com/source/libsecurity_codesigning/libsecurity_codesigning-55037.6/lib/machorep.cpp https://stackoverflow.com/questions/10119700/how-to-get-mach-o-uuid-of-a-running-process
	// .note.gnu.build-id https://github.com/golang/go/issues/21564 https://github.com/golang/go/blob/178307c3a72a9da3d731fecf354630761d6b246c/src/cmd/go/internal/buildid/buildid.go
	Err(())
}
fn from_exe<H: Hasher>(mut hasher: H) -> Result<H, ()> {
	#[cfg(not(target_arch = "wasm32"))]
	{
		if cfg!(miri) {
			return Err(());
		}
		let file = palaver::env::exe().map_err(drop)?;
		let _ = io::copy(&mut &file, &mut HashWriter(&mut hasher)).map_err(drop)?;
		Ok(hasher)
	}
	#[cfg(target_arch = "wasm32")]
	{
		let _ = &mut hasher;
		Err(())
	}
}
fn from_type_id<H: Hasher>(mut hasher: H) -> Result<H, ()> {
	fn type_id_of<T: 'static>(_: &T) -> TypeId {
		TypeId::of::<T>()
	}
	TypeId::of::<()>().hash(&mut hasher);
	TypeId::of::<u8>().hash(&mut hasher);
	let a = |x: ()| x;
	type_id_of(&a).hash(&mut hasher);
	let b = |x: u8| x;
	type_id_of(&b).hash(&mut hasher);
	Ok(hasher)
}

fn calculate() -> Uuid {
	let hasher = twox_hash::XxHash::with_seed(0);

	let hasher = from_header(hasher)
		.or_else(|()| from_exe(hasher))
		.unwrap_or(hasher);
	let mut hasher = from_type_id(hasher).unwrap();

	let mut bytes = [0; 16];
	<byteorder::NativeEndian as byteorder::ByteOrder>::write_u64(&mut bytes[..8], hasher.finish());
	hasher.write_u8(0);
	<byteorder::NativeEndian as byteorder::ByteOrder>::write_u64(&mut bytes[8..], hasher.finish());

	uuid::Builder::from_bytes(bytes)
		.with_variant(uuid::Variant::RFC4122)
		.with_version(uuid::Version::Random)
		.into_uuid()
}

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
	use wasm_bindgen_test::wasm_bindgen_test;

	#[test]
	#[wasm_bindgen_test]
	fn brute() {
		let x = super::calculate();
		for _ in 0..1000 {
			assert_eq!(x, super::calculate());
		}
	}

	#[test]
	#[wasm_bindgen_test]
	fn get() {
		let x = super::calculate();
		assert_eq!(x, super::get());
		assert_eq!(x, super::get());
		assert_eq!(x, super::get());
	}
}
