//! Timers and tickers: the runtime's clock (spec `docs/specs/reactive/timers.md`).
//!
//! [`timer`] runs a callback once after a delay; [`ticker`] runs one every tick while active.
//! Both belong to the owner current when created, like signals, and are driven by
//! [`Runtime::tick`](crate::Runtime::tick), which the app runner calls at the start of every
//! frame with the platform's clock.

use std::time::{Duration, Instant};

use tantu_core::Id;

use crate::runtime::RuntimeId;

/// A one-shot callback after a delay (REACTIVE-TIME-02). `Copy`; owned by the owner current
/// when created.
#[derive(Clone, Copy)]
pub struct Timer {
    #[allow(dead_code)]
    id: Id,
    #[allow(dead_code)]
    rt: RuntimeId,
}

/// Runs `f` once, at the first tick whose time is at least `delay` after now (the runtime's
/// time, REACTIVE-TIME-01).
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn timer(delay: Duration, f: impl FnOnce() + 'static) -> Timer {
    let _ = (delay, Box::new(f) as Box<dyn FnOnce()>);
    todo!()
}

impl Timer {
    /// Drops the callback without running it. Ignored once fired, cancelled or disposed.
    pub fn cancel(self) {
        todo!()
    }

    /// True until it fires, is cancelled or is disposed.
    pub fn is_pending(self) -> bool {
        todo!()
    }
}

/// A callback run once per tick while active, with the time since it started
/// (REACTIVE-TIME-03; Flutter's `Ticker`). `Copy`; owned by the owner current when created.
#[derive(Clone, Copy)]
pub struct Ticker {
    #[allow(dead_code)]
    id: Id,
    #[allow(dead_code)]
    rt: RuntimeId,
}

/// A ticker calling `f(elapsed)`; created stopped.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn ticker(f: impl FnMut(Duration) + 'static) -> Ticker {
    let _ = Box::new(f) as Box<dyn FnMut(Duration)>;
    todo!()
}

impl Ticker {
    /// Starts (or restarts) it: `elapsed` counts from now.
    pub fn start(self) {
        todo!()
    }

    /// Stops it; `f` isn't called until the next `start`.
    pub fn stop(self) {
        todo!()
    }

    /// True while started.
    pub fn is_active(self) -> bool {
        todo!()
    }
}

/// When the runtime next needs a tick (REACTIVE-TIME-05).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// A ticker is active (or a timer waits for the first tick): as soon as possible.
    Now,
    /// The earliest pending timer's deadline.
    At(Instant),
}
