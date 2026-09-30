//! [`Scope`], [`batch`], [`untrack`] and [`on_cleanup`].

/// An owner for nodes, disposed as a unit (for example one per element). `Copy`.
#[derive(Clone, Copy)]
pub struct Scope {}

impl Scope {
    /// A new scope owned by the current owner.
    ///
    /// # Panics
    ///
    /// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
    #[allow(clippy::new_without_default)]
    pub fn new() -> Scope {
        todo!()
    }

    /// Runs `f` with this scope as the owner of every node created inside it.
    ///
    /// # Panics
    ///
    /// If the scope is disposed.
    pub fn run<R>(self, f: impl FnOnce() -> R) -> R {
        let _ = f;
        todo!()
    }

    /// Disposes everything the scope owns, runs its cleanups, then disposes the scope itself.
    /// Does nothing if already disposed.
    pub fn dispose(self) {
        todo!()
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        todo!()
    }
}

/// Runs `f`; effects triggered by writes inside it run once, when the outermost `batch` returns.
/// Reads inside `f` see the new values. Without a current runtime it just calls `f`.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    let _ = f;
    todo!()
}

/// Runs `f` without subscribing the running computation to anything `f` reads.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    let _ = f;
    todo!()
}

/// Registers `f` to run when the current owner is disposed or, for an effect or memo, before it
/// re-runs.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    let _ = f;
    todo!()
}
