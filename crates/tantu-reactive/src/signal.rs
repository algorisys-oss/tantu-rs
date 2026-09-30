//! [`Signal`]: a readable, writable value.

use std::marker::PhantomData;

/// A readable, writable value. `Copy`, whatever `T` is.
pub struct Signal<T> {
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
        let _ = value;
        todo!()
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
        todo!()
    }

    /// Calls `f` with a reference to the value. Subscribes the running computation.
    ///
    /// # Panics
    ///
    /// As [`Signal::get`].
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        let _ = f;
        todo!()
    }

    /// [`Signal::get`] without subscribing.
    pub fn get_untracked(self) -> T
    where
        T: Clone,
    {
        todo!()
    }

    /// [`Signal::with`] without subscribing.
    pub fn with_untracked<R>(self, f: impl FnOnce(&T) -> R) -> R {
        let _ = f;
        todo!()
    }

    /// [`Signal::get`], or `None` if the signal is disposed.
    pub fn try_get(self) -> Option<T>
    where
        T: Clone,
    {
        todo!()
    }

    /// [`Signal::with`], or `None` if the signal is disposed.
    pub fn try_with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R> {
        let _ = f;
        todo!()
    }

    /// Replaces the value and notifies dependents. Does nothing if disposed.
    ///
    /// # Panics
    ///
    /// If the value is borrowed (a write inside the signal's own `with` or `update`).
    pub fn set(self, value: T) {
        let _ = value;
        todo!()
    }

    /// Changes the value in place and notifies dependents. Does nothing if disposed.
    ///
    /// # Panics
    ///
    /// As [`Signal::set`].
    pub fn update(self, f: impl FnOnce(&mut T)) {
        let _ = f;
        todo!()
    }

    /// Disposes the signal and drops its value. Does nothing if already disposed.
    pub fn dispose(self) {
        todo!()
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        todo!()
    }
}
