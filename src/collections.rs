//! `std::collections` shim.
//!
//! `HashMap` / `HashSet` route to `hashbrown` (the no_std equivalent
//! Rust's std HashMap is itself built on). `VecDeque`, `BTreeMap`,
//! `BTreeSet`, `BinaryHeap`, `LinkedList` re-export from `alloc::collections`.
//! Hash maps use hashbrown's default hasher (foldhash) rather than std's
//! `RandomState` because the latter needs OS-provided entropy at
//! initialisation.

pub use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
pub use hashbrown::{HashMap, HashSet};

pub mod hash_map {
    pub use hashbrown::hash_map::{
        DefaultHashBuilder, Drain, Entry, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys,
        OccupiedEntry, OccupiedError, RawEntryBuilder, RawEntryBuilderMut, RawEntryMut,
        RawOccupiedEntryMut, RawVacantEntryMut, VacantEntry, Values, ValuesMut,
    };
    pub use hashbrown::HashMap;
}

pub mod hash_set {
    pub use hashbrown::hash_set::{
        Difference, Drain, IntoIter, Intersection, Iter, SymmetricDifference, Union,
    };
    pub use hashbrown::HashSet;
}

pub mod btree_map {
    pub use alloc::collections::btree_map::*;
}

pub mod btree_set {
    pub use alloc::collections::btree_set::*;
}

pub mod vec_deque {
    pub use alloc::collections::vec_deque::*;
}

pub mod linked_list {
    pub use alloc::collections::linked_list::*;
}

pub mod binary_heap {
    pub use alloc::collections::binary_heap::*;
}
