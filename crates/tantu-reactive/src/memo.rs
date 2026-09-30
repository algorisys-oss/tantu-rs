//! [`Memo`]: a cached value derived from signals and other memos.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use tantu_core::Id;

use crate::runtime::{self, Kind, RuntimeId};

/// A cached value computed from other signals and memos. `Copy`, whatever `T` is.
///
/// Lazy: the computation runs on the first read, then only on a read after a dependency changed.
/// A recomputed value equal to the previous one doesn't notify dependents.
pub struct Memo<T> {
    id: Id,
    rt: RuntimeId,
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
    pub fn new(mut f: impl FnMut() -> T + 'static) -> Memo<T> {
        let inner = runtime::expect_current("memo");
        // `None` until the first computation.
        let value: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
        let slot = value.clone();
        let compute = move || {
            let new = f();
            let Ok(mut slot) = slot.try_borrow_mut() else {
                runtime::borrowed("Memo recompute");
            };
            if slot.as_ref() == Some(&new) {
                // Equal: keep the previous value, don't notify.
                false
            } else {
                *slot = Some(new);
                true
            }
        };
        let id = inner.create(
            Kind::Memo,
            Some(value),
            Some(Rc::new(RefCell::new(compute))),
        );
        Memo {
            id,
            rt: inner.id,
            ty: PhantomData,
        }
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
        self.with(T::clone)
    }

    /// Calls `f` with a reference to the up-to-date value. Subscribes the running computation.
    ///
    /// # Panics
    ///
    /// As [`Memo::get`].
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        self.read(true, f)
            .unwrap_or_else(|| runtime::disposed("Memo"))
    }

    /// [`Memo::get`] without subscribing.
    pub fn get_untracked(self) -> T
    where
        T: Clone,
    {
        self.with_untracked(T::clone)
    }

    /// [`Memo::with`] without subscribing.
    pub fn with_untracked<R>(self, f: impl FnOnce(&T) -> R) -> R {
        self.read(false, f)
            .unwrap_or_else(|| runtime::disposed("Memo"))
    }

    /// [`Memo::get`], or `None` if the memo is disposed.
    pub fn try_get(self) -> Option<T>
    where
        T: Clone,
    {
        self.try_with(T::clone)
    }

    /// [`Memo::with`], or `None` if the memo is disposed.
    pub fn try_with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.read(true, f)
    }

    /// Disposes the memo, its computation and what it owns. Does nothing if already disposed.
    pub fn dispose(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            if inner.is_alive(self.id, Kind::Memo) {
                inner.dispose(self.id);
            }
        }
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        runtime::lookup(self.rt).is_none_or(|inner| !inner.is_alive(self.id, Kind::Memo))
    }

    /// Brings the memo up to date and calls `f` with its value, or returns `None` if disposed.
    fn read<R>(self, track: bool, f: impl FnOnce(&T) -> R) -> Option<R> {
        let inner = runtime::lookup(self.rt)?;
        if !inner.is_alive(self.id, Kind::Memo) {
            return None;
        }
        inner.check_cycle(self.id);
        inner.update_if_necessary(self.id);
        // Subscribe after updating, so the first computation doesn't notify the reader.
        let cell = inner
            .value(self.id, track)?
            .downcast::<RefCell<Option<T>>>()
            .ok()?;
        let Ok(value) = cell.try_borrow() else {
            runtime::borrowed("Memo read");
        };
        value.as_ref().map(f)
    }
}
