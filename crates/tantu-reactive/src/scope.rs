//! [`Scope`], [`batch`], [`untrack`] and [`on_cleanup`].

use tantu_core::Id;

use crate::runtime::{self, Kind, RuntimeId};

/// An owner for nodes, disposed as a unit (for example one per element). `Copy`.
#[derive(Clone, Copy)]
pub struct Scope {
    id: Id,
    rt: RuntimeId,
}

impl Scope {
    /// A new scope owned by the current owner.
    ///
    /// # Panics
    ///
    /// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
    #[allow(clippy::new_without_default)]
    pub fn new() -> Scope {
        let inner = runtime::expect_current("Scope::new");
        let id = inner.create(Kind::Scope, None, None);
        Scope { id, rt: inner.id }
    }

    /// Runs `f` with this scope as the owner of every node created inside it.
    ///
    /// # Panics
    ///
    /// If the scope is disposed.
    pub fn run<R>(self, f: impl FnOnce() -> R) -> R {
        match runtime::lookup(self.rt) {
            Some(inner) => inner.run_in_scope(self.id, f),
            None => runtime::disposed("Scope::run: the scope"),
        }
    }

    /// Disposes everything the scope owns, runs its cleanups, then disposes the scope itself.
    /// Does nothing if already disposed.
    pub fn dispose(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            if inner.is_alive(self.id, Kind::Scope) {
                inner.dispose(self.id);
            }
        }
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        runtime::lookup(self.rt).is_none_or(|inner| !inner.is_alive(self.id, Kind::Scope))
    }
}

/// Runs `f`; effects triggered by writes inside it run once, when the outermost `batch` returns.
/// Reads inside `f` see the new values. Without a current runtime it just calls `f`.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    match runtime::current() {
        Some(inner) => inner.batch(f),
        None => f(),
    }
}

/// Runs `f` without subscribing the running computation to anything `f` reads.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    match runtime::current() {
        Some(inner) => inner.untrack(f),
        None => f(),
    }
}

/// Registers `f` to run when the current owner is disposed or, for an effect or memo, before it
/// re-runs.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    runtime::expect_current("on_cleanup").on_cleanup(Box::new(f));
}
