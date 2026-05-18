//! `std::prelude::v1` shim.
//!
//! Re-exports the same set as `core::prelude::v1` plus the alloc-level
//! types (Box, String, Vec, ToOwned, ToString) that std's prelude adds
//! beyond core.

pub mod v1 {
    pub use core::prelude::v1::*;
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
    pub use alloc::format;
    pub use alloc::vec;
}

pub use v1::*;
