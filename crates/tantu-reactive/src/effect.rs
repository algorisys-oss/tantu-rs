//! [`Effect`]: a computation run for its side effects.

/// A computation run for its side effects. `Copy`.
///
/// Runs once when created, then again whenever something it read in its latest run changes.
#[derive(Clone, Copy)]
pub struct Effect {}

/// Creates an effect owned by the current owner and runs it once, right away.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn effect(f: impl FnMut() + 'static) -> Effect {
    Effect::new(f)
}

impl Effect {
    /// Same as [`effect`].
    pub fn new(f: impl FnMut() + 'static) -> Effect {
        let _ = f;
        todo!()
    }

    /// Stops the effect for good, disposes what it owns and runs its cleanups. Does nothing if
    /// already disposed.
    pub fn dispose(self) {
        todo!()
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        todo!()
    }
}
