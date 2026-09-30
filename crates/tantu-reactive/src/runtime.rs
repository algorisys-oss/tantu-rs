//! [`Runtime`] and the reactive graph.

use std::marker::PhantomData;

/// The reactive graph: every signal, memo, effect and scope of one app or window.
///
/// Not `Send`. Handles resolve against the runtime that is current on this thread, set by
/// [`Runtime::enter`]. Dropping the runtime disposes every node it still holds.
pub struct Runtime {
    _not_send: PhantomData<*const ()>,
}

impl Runtime {
    /// An empty runtime.
    pub fn new() -> Runtime {
        todo!()
    }

    /// Runs `f` with this runtime as the current one on this thread, then restores the previous
    /// current runtime (also if `f` panics). Calls may nest.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _ = f;
        todo!()
    }

    /// Number of live nodes (signals, memos, effects and scopes). For leak tests.
    pub fn node_count(&self) -> usize {
        todo!()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new()
    }
}
