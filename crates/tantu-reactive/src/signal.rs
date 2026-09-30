//! [`Signal`]: a readable, writable value.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use tantu_core::Id;

use crate::runtime::{self, Inner, Kind, RuntimeId};

/// A readable, writable value. `Copy`, whatever `T` is.
pub struct Signal<T> {
    id: Id,
    rt: RuntimeId,
    ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}

/// Creates a signal in the current runtime, owned by the current owner.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn signal<T: 'static>(value: T) -> Signal<T> {
    Signal::new(value)
}

impl<T: 'static> Signal<T> {
    /// Same as [`signal`].
    pub fn new(value: T) -> Signal<T> {
        let inner = runtime::expect_current("signal");
        let id = inner.create(Kind::Signal, Some(Rc::new(RefCell::new(value))), None);
        Signal {
            id,
            rt: inner.id,
            ty: PhantomData,
        }
    }

    /// A clone of the value. Subscribes the running computation.
    ///
    /// # Panics
    ///
    /// If the signal is disposed, or its value is mutably borrowed (inside its own `update`).
    pub fn get(self) -> T
    where
        T: Clone,
    {
        self.with(T::clone)
    }

    /// Calls `f` with a reference to the value. Subscribes the running computation.
    ///
    /// # Panics
    ///
    /// As [`Signal::get`].
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        self.read(true, f)
            .unwrap_or_else(|| runtime::disposed("Signal"))
    }

    /// [`Signal::get`] without subscribing.
    pub fn get_untracked(self) -> T
    where
        T: Clone,
    {
        self.with_untracked(T::clone)
    }

    /// [`Signal::with`] without subscribing.
    pub fn with_untracked<R>(self, f: impl FnOnce(&T) -> R) -> R {
        self.read(false, f)
            .unwrap_or_else(|| runtime::disposed("Signal"))
    }

    /// [`Signal::get`], or `None` if the signal is disposed.
    pub fn try_get(self) -> Option<T>
    where
        T: Clone,
    {
        self.try_with(T::clone)
    }

    /// [`Signal::with`], or `None` if the signal is disposed.
    pub fn try_with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.read(true, f)
    }

    /// Replaces the value and notifies dependents. Does nothing if disposed.
    ///
    /// # Panics
    ///
    /// If the value is borrowed (a write inside the signal's own `with` or `update`).
    pub fn set(self, value: T) {
        self.update(|v| *v = value);
    }

    /// Changes the value in place and notifies dependents. Does nothing if disposed.
    ///
    /// # Panics
    ///
    /// As [`Signal::set`].
    pub fn update(self, f: impl FnOnce(&mut T)) {
        let Some((inner, cell)) = self.cell(false) else {
            return;
        };
        {
            let Ok(mut value) = cell.try_borrow_mut() else {
                runtime::borrowed("Signal::update");
            };
            f(&mut value);
        }
        inner.notify(self.id);
    }

    /// Disposes the signal and drops its value. Does nothing if already disposed.
    pub fn dispose(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            if inner.is_alive(self.id, Kind::Signal) {
                inner.dispose(self.id);
            }
        }
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        runtime::lookup(self.rt).is_none_or(|inner| !inner.is_alive(self.id, Kind::Signal))
    }

    /// The runtime and value cell, or `None` if disposed. Subscribes if `track`.
    fn cell(self, track: bool) -> Option<(Rc<Inner>, Rc<RefCell<T>>)> {
        let inner = runtime::lookup(self.rt)?;
        let cell = inner.value(self.id, track)?.downcast::<RefCell<T>>().ok()?;
        Some((inner, cell))
    }

    /// Calls `f` with the value, or returns `None` if disposed.
    fn read<R>(self, track: bool, f: impl FnOnce(&T) -> R) -> Option<R> {
        let (_, cell) = self.cell(track)?;
        let Ok(value) = cell.try_borrow() else {
            runtime::borrowed("Signal read");
        };
        Some(f(&value))
    }
}
