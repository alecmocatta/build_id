# build_id

[![Crates.io](https://img.shields.io/crates/v/build_id.svg?maxAge=86400)](https://crates.io/crates/build_id)
[![MIT / Apache 2.0 licensed](https://img.shields.io/crates/l/build_id.svg?maxAge=2592000)](#License)
[![Build Status](https://dev.azure.com/alecmocatta/build_id/_apis/build/status/tests?branchName=master)](https://dev.azure.com/alecmocatta/build_id/_build/latest?branchName=master)

[Docs](https://docs.rs/build_id/0.2.0)

Obtain a [`Uuid`](https://docs.rs/uuid/0.7/uuid/) uniquely representing the
build of the current binary.

This is intended to be used to check that different processes are indeed
invocations of identically laid out binaries.

As such:
* It is guaranteed to be identical within multiple invocations of the same
binary.
* It is guaranteed to be different across binaries with different code or data
segments or layout.
* Equality is unspecified if the binaries have identical code and data segments
and layout but differ immaterially (e.g. if a timestamp is included in the
binary at compile time).

## Examples

```rust
let local_build_id = build_id::get();
if local_build_id == remote_build_id {
	println!("We're running the same binary as remote!");
} else {
	println!("We're running a different binary to remote");
}
```

## Note

This looks first for linker-inserted build ID / binary UUIDs (i.e.
`.note.gnu.build-id` on Linux; `LC_UUID` in Mach-O; etc), falling back to
hashing the whole binary.

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
