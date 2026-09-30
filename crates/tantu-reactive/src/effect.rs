//! [`Effect`]: a computation run for its side effects.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::Id;

use crate::runtime::{self, Kind, RuntimeId};

/// A computation run for its side effects. `Copy`.
///
/// Runs once when created, then again whenever something it read in its latest run changes.
#[derive(Clone, Copy)]
pub struct Effect {
    id: Id,
    rt: RuntimeId,
}

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
    pub fn new(mut f: impl FnMut() + 'static) -> Effect {
        let inner = runtime::expect_current("effect");
        let compute = move || {
            f();
            true
        };
        let id = inner.create(Kind::Effect, None, Some(Rc::new(RefCell::new(compute))));
        // Created dirty: this runs it now, even inside a batch.
        inner.update_if_necessary(id);
        Effect { id, rt: inner.id }
    }

    /// Stops the effect for good, disposes what it owns and runs its cleanups. Does nothing if
    /// already disposed.
    pub fn dispose(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            if inner.is_alive(self.id, Kind::Effect) {
                inner.dispose(self.id);
            }
        }
    }

    /// True once disposed, or when used while its runtime is not current.
    pub fn is_disposed(self) -> bool {
        runtime::lookup(self.rt).is_none_or(|inner| !inner.is_alive(self.id, Kind::Effect))
    }
}
