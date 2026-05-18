//! `std` shim crate for macula-kernel.
//!
//! Re-exports `core::*` and `alloc::*` under `std::*` namespaces so that
//! standard-library-assuming crates (notably `quinn-proto`) can be compiled
//! against the kernel target unmodified. Where a path has no `core` or `alloc`
//! equivalent (`std::time::Instant`, `std::time::SystemTime`,
//! `std::sync::Mutex`, `std::collections::HashMap`, parts of `std::io`),
//! this crate provides a minimal shim. Some shims defer to the host kernel
//! via `extern "C"` symbols documented in `time.rs`.
//!
//! Pull this into a downstream crate with:
//!
//! ```ignore
//! extern crate macula_std as std;
//! ```
//!
//! at the top of `lib.rs`. Every `use std::X;` in the downstream crate then
//! resolves to this crate, including its sub-modules.

#![no_std]
#![allow(non_snake_case)]

extern crate alloc;

pub use core::any;
pub use core::ascii;
pub use core::array;
pub use core::cell;
pub use core::char;
pub use core::clone;
pub use core::cmp;
pub use core::convert;
pub use core::default;
pub use core::f32;
pub use core::f64;
pub use core::ffi;
pub use core::fmt;
pub use core::future;
pub use core::hash;
pub use core::hint;
pub use core::i128;
pub use core::i16;
pub use core::i32;
pub use core::i64;
pub use core::i8;
pub use core::isize;
pub use core::iter;
pub use core::marker;
pub use core::mem;
pub use core::net;
pub use core::num;
pub use core::ops;
pub use core::option;
pub use core::panic;
pub use core::pin;
pub use core::primitive;
pub use core::ptr;
pub use core::result;
pub use core::slice;
pub use core::str;
pub use core::task;
pub use core::u128;
pub use core::u16;
pub use core::u32;
pub use core::u64;
pub use core::u8;
pub use core::usize;

pub use alloc::borrow;
pub use alloc::boxed;
pub use alloc::format;
pub use alloc::rc;
pub use alloc::string;
pub use alloc::vec;

pub use core::{
    assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne,
    matches, write, writeln,
};

pub use alloc::{format as _format_helper};

pub mod collections;
pub mod io;
pub mod prelude;
pub mod sync;
pub mod time;

/// Error type re-exports (the `std::error::Error` trait moved into `core` on
/// recent Rust, but downstream crates spell it `std::error::Error`).
pub mod error {
    pub use core::error::Error;
}

/// `std::process` shim. Only `abort()` is supplied; downstream crates use it
/// in the same places they would call `panic!()` when running freestanding.
pub mod process {
    /// Abort the current execution. We translate to a panic so the kernel's
    /// panic handler runs; there is no separate process boundary to abort.
    pub fn abort() -> ! {
        panic!("std::process::abort()");
    }
}

/// Re-export `Cow` at `std::borrow::Cow` (alloc already places it there, but
/// some crates spell it `std::borrow::Cow`).
pub use alloc::borrow::{Cow, ToOwned};
