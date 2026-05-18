//! `std::prelude::v1` shim.
//!
//! Re-exports the same set as `core::prelude::v1` plus the alloc-level
//! types (Box, String, Vec, ToOwned, ToString) that std's prelude adds
//! beyond core.

pub mod v1 {
    // core::prelude::v1 is the 2015-edition prelude. To match real std's
    // 2021 prelude (TryFrom, TryInto, FromIterator are members), pull from
    // rust_2021 instead. Note: rust_2024 will exist eventually; bump when
    // downstream crates start using edition 2024 idioms.
    pub use core::prelude::rust_2021::*;
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
    pub use alloc::format;
    pub use alloc::vec;
}

pub use v1::*;
