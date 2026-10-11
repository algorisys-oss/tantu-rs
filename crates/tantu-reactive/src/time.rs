//! Timers and tickers: the runtime's clock (spec `docs/specs/reactive/timers.md`).
//!
//! [`timer`] runs a callback once after a delay; [`ticker`] runs one every tick while active.
//! Both belong to the owner current when created, like signals, and are driven by
//! [`Runtime::tick`](crate::Runtime::tick), which the app runner calls at the start of every
//! frame with the platform's clock.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use tantu_core::Id;

use crate::runtime::{self, Kind, RuntimeId};

/// The state behind a [`Timer`] node.
pub(crate) struct TimerState {
    pub(crate) delay: Duration,
    /// Resolved at creation when the clock has ticked, else at the first tick.
    pub(crate) deadline: Cell<Option<Instant>>,
    /// Taken out to run, or by `cancel`.
    pub(crate) f: RefCell<Option<Box<dyn FnOnce()>>>,
    /// Creation order, to order timers with equal deadlines.
    pub(crate) seq: u64,
}

/// A ticker's callback.
type TickerFn = Box<dyn FnMut(Duration)>;

/// The state behind a [`Ticker`] node.
pub(crate) struct TickerState {
    /// Taken out while it runs, so the callback can stop or restart its own ticker.
    pub(crate) f: RefCell<Option<TickerFn>>,
    /// `None` while stopped; `Some(None)` once started before the first tick.
    pub(crate) start: Cell<Option<Option<Instant>>>,
}

/// A one-shot callback after a delay (REACTIVE-TIME-02). `Copy`; owned by the owner current
/// when created.
#[derive(Clone, Copy)]
pub struct Timer {
    id: Id,
    rt: RuntimeId,
}

/// Runs `f` once, at the first tick whose time is at least `delay` after now (the runtime's
/// time, REACTIVE-TIME-01).
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn timer(delay: Duration, f: impl FnOnce() + 'static) -> Timer {
    let inner = runtime::expect_current("timer");
    let id = inner.create_timer(delay, Box::new(f));
    Timer { id, rt: inner.id }
}

impl Timer {
    /// Drops the callback without running it. Ignored once fired, cancelled or disposed.
    pub fn cancel(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            if inner.is_alive(self.id, Kind::Timer) {
                inner.dispose(self.id);
            }
        }
    }

    /// True until it fires, is cancelled or is disposed.
    pub fn is_pending(self) -> bool {
        runtime::lookup(self.rt).is_some_and(|inner| inner.is_alive(self.id, Kind::Timer))
    }
}

/// A callback run once per tick while active, with the time since it started
/// (REACTIVE-TIME-03; Flutter's `Ticker`). `Copy`; owned by the owner current when created.
#[derive(Clone, Copy)]
pub struct Ticker {
    id: Id,
    rt: RuntimeId,
}

/// A ticker calling `f(elapsed)`; created stopped.
///
/// # Panics
///
/// If no runtime is current (see [`Runtime::enter`](crate::Runtime::enter)).
pub fn ticker(f: impl FnMut(Duration) + 'static) -> Ticker {
    let inner = runtime::expect_current("ticker");
    let id = inner.create_ticker(Box::new(f));
    Ticker { id, rt: inner.id }
}

impl Ticker {
    /// Starts (or restarts) it: `elapsed` counts from now.
    pub fn start(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            inner.set_ticker_active(self.id, true);
        }
    }

    /// Stops it; `f` isn't called until the next `start`.
    pub fn stop(self) {
        if let Some(inner) = runtime::lookup(self.rt) {
            inner.set_ticker_active(self.id, false);
        }
    }

    /// True while started.
    pub fn is_active(self) -> bool {
        runtime::lookup(self.rt).is_some_and(|inner| {
            inner
                .ticker_state(self.id)
                .is_some_and(|state| state.start.get().is_some())
        })
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

impl runtime::Inner {
    /// Adds a timer node owned by the current owner (REACTIVE-TIME-02).
    pub(crate) fn create_timer(&self, delay: Duration, f: Box<dyn FnOnce()>) -> Id {
        let (deadline, seq) = self.clock_for_new();
        let state = Rc::new(TimerState {
            delay,
            deadline: Cell::new(deadline.map(|now| now + delay)),
            f: RefCell::new(Some(f)),
            seq,
        });
        let id = self.create(Kind::Timer, Some(state), None);
        self.register_clock_node(id, Kind::Timer);
        id
    }

    /// Adds a ticker node owned by the current owner, stopped (REACTIVE-TIME-03).
    pub(crate) fn create_ticker(&self, f: TickerFn) -> Id {
        let state = Rc::new(TickerState {
            f: RefCell::new(Some(f)),
            start: Cell::new(None),
        });
        let id = self.create(Kind::Ticker, Some(state), None);
        self.register_clock_node(id, Kind::Ticker);
        id
    }

    /// The timer state of a live timer node.
    pub(crate) fn timer_state(&self, id: Id) -> Option<Rc<TimerState>> {
        self.value(id, false)?.downcast::<TimerState>().ok()
    }

    /// The ticker state of a live ticker node.
    pub(crate) fn ticker_state(&self, id: Id) -> Option<Rc<TickerState>> {
        self.value(id, false)?.downcast::<TickerState>().ok()
    }

    /// Starts or stops a ticker (ignored for a dead handle).
    pub(crate) fn set_ticker_active(&self, id: Id, active: bool) {
        let Some(state) = self.ticker_state(id) else {
            return;
        };
        state
            .start
            .set(if active { Some(self.now()) } else { None });
    }

    /// Runs what is due at `now` (REACTIVE-TIME-01): due timers (earliest first), then every
    /// active ticker once. Callbacks run untracked, with the node's owner current, inside one
    /// batch.
    pub(crate) fn tick(&self, now: Instant) -> usize {
        self.batch(|| {
            let (due, tickers) = self.begin_tick(now);
            let mut ran = 0;
            for id in due {
                let Some(state) = self.timer_state(id) else {
                    continue;
                };
                let f = state.f.borrow_mut().take();
                let owner = self.owner_of(id);
                self.dispose(id);
                if let Some(f) = f {
                    self.with_owner(owner, || self.untrack(f));
                    ran += 1;
                }
            }
            for id in tickers {
                let Some(state) = self.ticker_state(id) else {
                    continue;
                };
                let Some(start) = state.start.get() else {
                    continue;
                };
                let start = match start {
                    Some(start) => start,
                    None => {
                        state.start.set(Some(Some(now)));
                        now
                    }
                };
                let f = state.f.borrow_mut().take();
                if let Some(mut f) = f {
                    let owner = self.owner_of(id);
                    self.with_owner(owner, || self.untrack(|| f(now - start)));
                    ran += 1;
                    if self.is_alive(id, Kind::Ticker) {
                        *state.f.borrow_mut() = Some(f);
                    }
                }
            }
            ran
        })
    }

    /// `Wake::Now` while a ticker is active or a timer has no deadline yet, else the earliest
    /// deadline (REACTIVE-TIME-05).
    pub(crate) fn next_wake(&self) -> Option<Wake> {
        let (timers, tickers) = self.clock_nodes();
        for id in tickers {
            if self
                .ticker_state(id)
                .is_some_and(|s| s.start.get().is_some())
            {
                return Some(Wake::Now);
            }
        }
        let mut earliest: Option<Instant> = None;
        for id in timers {
            let Some(state) = self.timer_state(id) else {
                continue;
            };
            match state.deadline.get() {
                None => return Some(Wake::Now),
                Some(deadline) => {
                    earliest = Some(earliest.map_or(deadline, |e| e.min(deadline)));
                }
            }
        }
        earliest.map(Wake::At)
    }
}
