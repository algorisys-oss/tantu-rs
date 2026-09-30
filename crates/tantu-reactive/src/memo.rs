//! [`Memo`]: a cached value derived from signals and other memos.

use std::marker::PhantomData;

/// A cached value computed from other signals and memos. `Copy`, whatever `T` is.
///
/// Lazy: the computation runs on the first read, then only on a read after a dependency changed.
/// A recomputed value equal to the previous one doesn't notify dependents.
pub struct Memo<T> {
    ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Memo<T> {}

/// Creates a memo owned by the current owner. `f` doesn't run until the first read.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn memo<T: PartialEq + 'static>(f: impl FnMut() -> T + 'static) -> Memo<T> {
    Memo::new(f)
}

impl<T: PartialEq + 'static> Memo<T> {
    /// Same as [`memo`].
    pub fn new(f: impl FnMut() -> T + 'static) -> Memo<T> {
        let _ = f;
        todo!()
    }

    /// A clone of the up-to-date value. Subscribes the running computation.
    ///
    /// # Panics
    ///
    /// If the memo is disposed, or reads itself (a cycle).
    pub fn get(self) -> T
    where
        T: Clone,
    {
        todo!()
    }

    /// Calls `f` with a reference to the up-to-date value. Subscribes the running computation.
    ///
    /// # Panics
    ///
    /// As [`Memo::get`].
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        let _ = f;
        todo!()
    }

    /// [`Memo::get`] without subscribing.
    pub fn get_untracked(self) -> T
    where
        T: Clone,
    {
        todo!()
    }

    /// [`Memo::with`] without subscribing.
    pub fn with_untracked<R>(self, f: impl FnOnce(&T) -> R) -> R {
        let _ = f;
        todo!()
    }

    /// [`Memo::get`], or `None` if the memo is disposed.
    pub fn try_get(self) -> Option<T>
    where
        T: Clone,
    {
        todo!()
    }

    /// [`Memo::with`], or `None` if the memo is disposed.
    pub fn try_with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R> {
        let _ = f;
        todo!()
    }

    /// Disposes the memo, its computation and what it owns. Does nothing if already disposed.
    pub fn dispose(self) {
        todo!()
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        todo!()
    }
}
