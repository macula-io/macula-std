//! `std::sync` shim.
//!
//! `Arc` re-exports `alloc::sync::Arc` unchanged. `Mutex` / `RwLock` /
//! `Once` wrap `spin::*` so the API mirrors `std::sync` but avoids OS
//! mutex primitives.

pub use alloc::sync::{Arc, Weak};
pub use core::sync::atomic;

use core::fmt;

/// Poison-free `Mutex` backed by `spin::Mutex`. The API matches
/// `std::sync::Mutex` minus the `Result<_, PoisonError<_>>` wrapping
/// because `spin::Mutex` panics on poisoning rather than tracking it.
pub struct Mutex<T: ?Sized> {
    inner: spin::Mutex<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    inner: spin::MutexGuard<'a, T>,
}

#[derive(Debug)]
pub struct PoisonError<T>(T);

pub type LockResult<G> = Result<G, PoisonError<G>>;
pub type TryLockResult<G> = Result<G, TryLockError<G>>;

#[derive(Debug)]
pub enum TryLockError<T> {
    Poisoned(PoisonError<T>),
    WouldBlock,
}

impl<T> PoisonError<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
    pub fn get_ref(&self) -> &T {
        &self.0
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Self {
        Self { inner: spin::Mutex::new(t) }
    }
    pub fn into_inner(self) -> LockResult<T> {
        Ok(self.inner.into_inner())
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        Ok(MutexGuard { inner: self.inner.lock() })
    }
    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        self.inner
            .try_lock()
            .map(|g| MutexGuard { inner: g })
            .ok_or(TryLockError::WouldBlock)
    }
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        Ok(self.inner.get_mut())
    }
}

impl<'a, T: ?Sized> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T: ?Sized> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.inner
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mutex").finish_non_exhaustive()
    }
}

/// RwLock backed by `spin::RwLock`. Same poison-free shape as Mutex.
pub struct RwLock<T: ?Sized> {
    inner: spin::RwLock<T>,
}

pub struct RwLockReadGuard<'a, T: ?Sized + 'a> {
    inner: spin::RwLockReadGuard<'a, T>,
}

pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    inner: spin::RwLockWriteGuard<'a, T>,
}

impl<T> RwLock<T> {
    pub const fn new(t: T) -> Self {
        Self { inner: spin::RwLock::new(t) }
    }
    pub fn into_inner(self) -> LockResult<T> {
        Ok(self.inner.into_inner())
    }
}

impl<T: ?Sized> RwLock<T> {
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        Ok(RwLockReadGuard { inner: self.inner.read() })
    }
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        Ok(RwLockWriteGuard { inner: self.inner.write() })
    }
    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        self.inner
            .try_read()
            .map(|g| RwLockReadGuard { inner: g })
            .ok_or(TryLockError::WouldBlock)
    }
    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T>> {
        self.inner
            .try_write()
            .map(|g| RwLockWriteGuard { inner: g })
            .ok_or(TryLockError::WouldBlock)
    }
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        Ok(self.inner.get_mut())
    }
}

impl<'a, T: ?Sized> core::ops::Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T: ?Sized> core::ops::Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T: ?Sized> core::ops::DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.inner
    }
}

/// `Once` backed by `spin::Once`. Same API as `std::sync::Once`.
pub struct Once {
    inner: spin::Once<()>,
}

impl Once {
    pub const fn new() -> Self {
        Self { inner: spin::Once::new() }
    }
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        self.inner.call_once(|| f());
    }
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed()
    }
}
