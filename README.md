# macula-std

`std` shim crate for macula-kernel. Lets std-only Rust crates (notably
[`quinn-proto`](https://crates.io/crates/quinn-proto)) compile against
the `x86_64-macula` kernel target without source modifications, by
re-exporting `core::*` and `alloc::*` under `std::*` namespaces and
providing minimal shims for the types that have no `core` / `alloc`
equivalent.

## How to use

Pull this crate into the std-only downstream crate and add at the top of
its `lib.rs`:

```rust
extern crate alloc;
extern crate macula_std as std;
```

That single line makes every `use std::*` in the downstream crate
resolve to this crate, including its sub-modules. No file-by-file
patching needed.

## Shimmed types

The "re-exports" column is direct passthrough. The "shimmed" column is
this crate's own implementation matching the `std` API surface.

| Path | Backing |
|---|---|
| `std::any`, `std::cmp`, `std::convert`, `std::fmt`, `std::hash`, `std::iter`, `std::marker`, `std::mem`, `std::ops`, `std::option`, `std::result`, `std::slice`, `std::str`, `std::net` | `core` (re-export) |
| `std::borrow`, `std::boxed`, `std::format`, `std::rc`, `std::string`, `std::vec` | `alloc` (re-export) |
| `std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque}` | `alloc::collections` (re-export) |
| `std::collections::{HashMap, HashSet}` and `hash_map::Entry` | [`hashbrown`](https://crates.io/crates/hashbrown) |
| `std::time::Duration` | `core::time::Duration` (re-export) |
| `std::time::{Instant, SystemTime, UNIX_EPOCH}` | shim, calls `extern "C" fn macula_std_monotonic_ns/wallclock_ns` (host kernel provides) |
| `std::sync::{Arc, Weak}` | `alloc::sync::*` (re-export) |
| `std::sync::{Mutex, RwLock, Once}` | shim, wraps `spin::*` |
| `std::sync::atomic::*` | `core::sync::atomic::*` (re-export) |
| `std::io::{Error, ErrorKind, Result, Read, Write, Seek}` | shim, traits only, no OS-error backing |
| `std::error::Error` | `core::error::Error` (re-export, stabilised in Rust 1.81) |
| `std::prelude::v1::*` | shim, `core::prelude::v1` + alloc adds |

## What the host kernel must provide

The `Instant` and `SystemTime` shims defer to two `extern "C"` symbols
the host kernel must define:

```rust
#[no_mangle]
extern "C" fn macula_std_monotonic_ns() -> u64 {
    // returns nanoseconds since boot (or any monotonic epoch)
}

#[no_mangle]
extern "C" fn macula_std_wallclock_ns() -> u64 {
    // returns nanoseconds since Unix epoch
    // may return 0 if no RTC; downstream TLS cert checks will then
    // correctly fail all time-dependent validations
}
```

In `macula-kernel`, define these in a single small module that wires
into your existing TSC + RTC drivers.

## What this crate does NOT shim

- `std::process` (no processes in a kernel)
- `std::env` (no environment vars)
- `std::fs` (no filesystem at this layer; use kernel VFS directly)
- `std::path` (depends on filesystem semantics)
- `std::thread::*` (kernel scheduling is done elsewhere)
- `std::os::unix::*` / `std::os::*` (no POSIX surface in the kernel)
- `std::backtrace` (no DWARF unwinder)
- `std::panic::*` (kernel has its own panic handler)
- `std::eprintln!` / `std::println!` macros (kernel uses `log::info!` etc.)

If a downstream crate hits an unshimmed path, the fix is either to
add it here (one PR per type) or to patch the downstream crate to
avoid that path.

## License

MIT OR Apache-2.0 (the shim code, this README, the manifest).

Re-exports inherit their source crate's licenses (`core`, `alloc`,
`hashbrown`, `spin`). All MIT / Apache-2.0 compatible.
